use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::bindings::{Binding, BindingStack, VariableState, BindingManager};
use crate::counter::Counter;
use crate::error::PolarResult;
use crate::events::QueryEvent;
use crate::folder::{fold_value, Folder};
use crate::formatting::ToPolarString;
use crate::kb::Bindings;
use crate::partial::simplify_bindings;
use crate::runnable::Runnable;
use crate::terms::{Operation, Operator, Symbol, Term, Value};
use crate::vm::{cycle_constraints, Goals, PolarVirtualMachine};

#[derive(Clone)]
pub struct Inverter {
    vm: PolarVirtualMachine,
    bindings: Rc<RefCell<BindingStack>>,
    bsp: usize,
    results: Vec<BindingStack>,
    results_binding_managers: Vec<BindingManager>,
    add_constraints: Rc<RefCell<Bindings>>,
    _debug_id: u64,
    follower: Option<usize>
}

static ID: AtomicU64 = AtomicU64::new(0);

impl Inverter {
    pub fn new(
        vm: &PolarVirtualMachine,
        goals: Goals,
        bindings: Rc<RefCell<BindingStack>>,
        add_constraints: Rc<RefCell<Bindings>>,
        bsp: usize,
    ) -> Self {
        let mut vm = vm.clone_with_goals(goals);
        vm.inverting = true;
        Self {
            vm,
            bindings,
            bsp,
            add_constraints,
            results: vec![],
            results_binding_managers: vec![],
            _debug_id: ID.fetch_add(1, Ordering::AcqRel),
            follower: None
        }
    }
}

struct PartialInverter {
    this_var: Symbol,
    old_state: VariableState,
}

impl PartialInverter {
    pub fn new(this_var: Symbol, old_state: VariableState) -> Self {
        Self {
            this_var,
            old_state,
        }
    }

    fn invert_operation(&self, o: &Operation) -> Operation {
        // Compute csp from old_value vs. p.
        let csp = match &self.old_state {
            VariableState::Partial(e) => e.constraints().len(),
            _ => 0,
        };
        o.clone_with_constraints(o.inverted_constraints(csp))
    }
}

impl Folder for PartialInverter {
    /// Invert top-level constraints.
    fn fold_term(&mut self, t: Term) -> Term {
        t.clone_with_value(match t.value() {
            Value::Expression(o) => Value::Expression(self.invert_operation(o)),
            v => Value::Expression(op!(
                And,
                term!(value!(op!(
                    Neq,
                    term!(value!(self.this_var.clone())),
                    term!(fold_value(v.clone(), self))
                )))
            )),
        })
    }
}

fn results_to_constraints(results: Vec<BindingManager>) -> Bindings {
    let inverted = results
        .into_iter()
        .map(|bindings| invert_partials_bm(bindings))
        .collect();

    // Now have disjunction of results. not OR[result1, result2, ...]
    // Reduce constraints converts it into a conjunct of negated results.
    // AND[!result1, ...]
    let reduced = reduce_constraints_bm(inverted);
    let simplified =
        simplify_bindings(reduced.clone()).unwrap_or_else(Bindings::new);

    // TODO this logic is similar to get constraints in binding manager.
    simplified.into_iter().map(|(k, v)| {
        match v.value() {
            Value::Expression(_) => (k, v),
            _ => (k.clone(), v.clone_with_value(Value::Expression(op!(Unify, term!(k), v.clone()))))
        }
    }).collect()
}


/// Invert partial values in `bindings` with respect to the old VM bindings.
fn invert_partials(bindings: BindingStack, vm: &PolarVirtualMachine, bsp: usize) -> BindingStack {
    let mut new_bindings = vec![];
    for Binding(var, value) in bindings {
        // Determine whether to invert partials based on the state of the variable in the VM
        // before inversion.
        match vm.variable_state_at_point(&var, bsp) {
            // TODO: This case is for something like
            // w(x) if not (y = 1) and y = x;
            //
            // Ultimately this should add constraints, but for now this query always succeeds with
            // no constraints because a negation is performed over a variable that is not
            // bound.
            VariableState::Unbound => (),
            // during the negation to a different value (the negated query would backtrack).
            VariableState::Bound(x) => assert_eq!(x, value, "inconsistent bindings"),
            VariableState::Cycle(c) => {
                let constraints =
                    PartialInverter::new(var.clone(), VariableState::Cycle(c.clone()))
                        .fold_term(value);

                // TODO (dhatch): This whole thing could just be an add constraints call.
                match constraints.value() {
                    Value::Expression(e) => {
                        let mut f = cycle_constraints(c);
                        f.merge_constraints(e.clone());
                        for var in f.variables() {
                            new_bindings.push(Binding(var.clone(), f.clone().into_term()));
                        }
                    }
                    _ => unreachable!("Constraint from partial inverter must be expression."),
                }
            }
            // Three states of a partial x
            //
            // x > 1 and not (x < 0)
            //
            // - before inversion partial (two constraints)
            // - post-inversion but pre-simplification partial (>2 constraints)
            // - post-inversion post-simplification partial (>2 constraints)
            //
            VariableState::Partial(e) => {
                let constraints =
                    PartialInverter::new(var.clone(), VariableState::Partial(e.clone()))
                        .fold_term(value);
                // Same thing here.
                match constraints.value() {
                    Value::Expression(f) => {
                        let mut e = e.clone();
                        e.merge_constraints(f.clone());
                        for var in e.variables() {
                            new_bindings.push(Binding(var.clone(), e.clone().into_term()));
                        }
                    }
                    _ => unreachable!("Constraint from partial inverter must be expression."),
                }
            }
        }
    }
    new_bindings
}

fn invert_partials_bm(bindings: BindingManager) -> Bindings {
     let mut new_bindings = Bindings::new();

     for var in bindings.variables() {
         let constraint = bindings.get_constraints(&var);
         new_bindings.insert(var.clone(), term!(constraint));
     }
    println!("1 before simplified: ");
    for (constraint, val) in new_bindings.iter() {
        println!("{:?} {}", constraint, val.to_polar());
    }

    //let new_bindings = bindings.bindings(true);

    let simplified =
        simplify_bindings(new_bindings.clone()).unwrap_or_else(Bindings::new);
    println!("1 simplified: ");
    for (constraint, val) in simplified.iter() {
        println!("{:?} {}", constraint, val.to_polar());
    }

    let inverted = simplified.into_iter().filter_map(|(k, v)| {
        match v.value() {
            Value::Expression(e) => Some((k, e.clone_with_constraints(e.inverted_constraints(0)).into_term())),
            _ => Some((k.clone(), term!(op!(And, term!(op!(Neq, term!(k), v.clone())))))),
        }
    }).collect::<Bindings>();

    println!("1 inverted: ");
    for (constraint, val) in inverted.iter() {
        println!("{:?} {}", constraint, val.to_polar());
    }

    inverted
}

/// Reduce + merge constraints.
fn reduce_constraints_bm(bindings: Vec<Bindings>) -> Bindings {
    let reduced = bindings
        .into_iter()
        .fold(Bindings::new(), |mut acc, bindings| {
            bindings
                .into_iter()
                .for_each(|(var, value)| match acc.entry(var.clone()) {
                    Entry::Occupied(mut o) => match (o.get().value(), value.value()) {
                        (Value::Expression(x), Value::Expression(y)) => {
                            let mut x = x.clone();
                            x.merge_constraints(y.clone());
                            o.insert(value.clone_with_value(value!(x)));
                        }
                        (existing, new) => panic!(
                            "Illegal state reached while reducing constraints for {}: {} → {}",
                            var,
                            existing.to_polar(),
                            new.to_polar()
                        ),
                    },
                    Entry::Vacant(v) => {
                        v.insert(value);
                    }
                });
            acc
        });
    reduced
}


// TODO Remove dedupe bindings once inverter operates over BindingManager instead of binding stack.
/// Only keep latest bindings.
fn dedupe_bindings(bindings: BindingStack) -> BindingStack {
    let mut seen = HashSet::new();
    bindings
        .into_iter()
        .rev()
        .filter(|Binding(v, _)| seen.insert(v.clone()))
        .collect()
}


/// Reduce + merge constraints.
fn reduce_constraints(bindings: Vec<BindingStack>) -> (Bindings, Vec<Symbol>) {
    let mut vars = vec![];
    let reduced = bindings
        .into_iter()
        .fold(Bindings::new(), |mut acc, bindings| {
            dedupe_bindings(bindings)
                .into_iter()
                .for_each(|Binding(var, value)| match acc.entry(var.clone()) {
                    Entry::Occupied(mut o) => match (o.get().value(), value.value()) {
                        (Value::Expression(x), Value::Expression(y)) => {
                            let mut x = x.clone();
                            x.merge_constraints(y.clone());
                            o.insert(value.clone_with_value(value!(x)));
                        }
                        (existing, new) => panic!(
                            "Illegal state reached while reducing constraints for {}: {} → {}",
                            var,
                            existing.to_polar(),
                            new.to_polar()
                        ),
                    },
                    Entry::Vacant(v) => {
                        v.insert(value);
                        vars.push(var);
                    }
                });
            acc
        });
    (reduced, vars)
}

/// A Runnable that runs a query and inverts the results in three ways:
///
/// 1. If no results are emitted (indicating failure), return true.
/// 2. If at least one result is emitted containing a partial, invert the partial's constraints,
///    pass the inverted partials back to the parent Runnable via a shared BindingStack, and return
///    true.
/// 3. In all other cases, return false.
impl Runnable for Inverter {
    fn run(&mut self, _: Option<&mut Counter>) -> PolarResult<QueryEvent> {
        if self.follower.is_none() {
            self.follower = Some(self.vm.add_binding_follower());
            println!("{} added follower {}", self._debug_id, self.follower.unwrap());
        }

        loop {
            // Pass most events through, but collect results and invert them.
            match self.vm.run(None)? {
                QueryEvent::Done { .. } => {
                    let mut result = self.results.is_empty();
                    if !result {
                        // If there are results, the inversion should usually fail. However,
                        // if those results have constraints we collect them and pass them
                        // out to the parent VM.
                        let constraints = results_to_constraints(self.results_binding_managers.drain(..).collect::<Vec<_>>());

                        // Decide which variables come out of negation. This is hacky.
                        // And the unbound one should sometimes come out I think...
                        // HACK: Remove vars with _value.
                        let constraints = constraints.into_iter().filter(|(k, _)| !(
                                k.0.starts_with("_value") || k.0.starts_with("_runnable")
                        ) && self.vm.variable_state_at_point(k, self.bsp) != VariableState::Unbound).collect::<Bindings>();
                        println!("inverter constraints: ");
                        for (constraint, val) in constraints.iter() {
                            println!("{:?} {}", constraint, val.to_polar());
                        }

                        // OLD
                        let inverted: Vec<BindingStack> = self
                            .results
                            .drain(..)
                            .collect::<Vec<BindingStack>>()
                            .into_iter()
                            // Inverts each result
                            .map(|bindings| invert_partials(bindings, &self.vm, self.bsp))
                            .collect();

                        // Now have disjunction of results. not OR[result1, result2, ...]
                        // Reduce constraints converts it into a conjunct of negated results.
                        // AND[!result1, ...]
                        let (reduced, ordered_vars) = reduce_constraints(inverted);
                        let simplified =
                            simplify_bindings(reduced.clone()).unwrap_or_else(Bindings::new);

                        let simplified_keys = simplified.keys().collect::<HashSet<&Symbol>>();
                        let reduced_keys = reduced.keys().collect::<HashSet<&Symbol>>();
                        let ordered_keys = ordered_vars.iter().collect::<HashSet<&Symbol>>();

                        assert_eq!(simplified_keys, reduced_keys);
                        assert_eq!(reduced_keys, ordered_keys);

                        // Figure out which bindings should go into parent VM's binding
                        // stack.
                        let new_bindings = ordered_vars.into_iter().flat_map(|var| {
                            // We have at least one binding to return, so succeed.
                            // result = true;

                            let value = simplified[&var].clone();
                            if let Value::Expression(_) = value.value() {
                                match self.vm.variable_state_at_point(&var, self.bsp) {
                                    VariableState::Unbound => vec![Binding(var, value)],
                                    VariableState::Bound(x) => {
                                        assert_eq!(x, value, "inconsistent bindings");
                                        vec![Binding(var, value)]
                                    }
                                    VariableState::Cycle(c) => {
                                        let constraint = cycle_constraints(c.clone())
                                            .clone_with_new_constraint(value)
                                            .into_term();
                                        c.into_iter()
                                            .map(|v| Binding(v, constraint.clone()))
                                            .collect()
                                    }
                                    VariableState::Partial(e) => {
                                        let e = e.clone_with_new_constraint(value);
                                        e.variables()
                                            .into_iter()
                                            .map(|var| Binding(var, e.clone().into_term()))
                                            .collect()
                                    }
                                }
                            } else {
                                vec![Binding(var, value)]
                            }
                        });

                        println!("{} new bindings: ", self._debug_id);
                        for Binding(var, value) in new_bindings {
                            println!("{:?} {}", var, value.to_polar());
                        }

                        // Return new bindings created by inverter to parent VM.
                        // OLD
                        if !constraints.is_empty() {
                            result = true;
                        }
                        self.add_constraints.borrow_mut().extend(constraints);
                    }
                    println!("{} done {}", self._debug_id, result);
                    return Ok(QueryEvent::Done { result });
                }
                QueryEvent::Result { .. } => {
                    // Retrieve new bindings made when running inverted query.
                    // Bindings are retrieved as the raw BindingStack.
                    let bindings: BindingStack =
                        self.vm.bindings_for_inverter().drain(self.bsp..).collect();
                    // Add new part of binding stack from inversion to results.
                    self.results.push(bindings);

                    let binding_follower = self.vm.remove_binding_follower(&self.follower.unwrap()).unwrap();
                    println!("{} removed follower {}", self._debug_id, self.follower.unwrap());
                    self.results_binding_managers.push(binding_follower);
                    self.follower = Some(self.vm.add_binding_follower());
                    println!("{} added follower {}", self._debug_id, self.follower.unwrap());
                }
                event => return Ok(event),
            }
        }
    }

    fn external_question_result(&mut self, call_id: u64, answer: bool) -> PolarResult<()> {
        self.vm.external_question_result(call_id, answer)
    }

    fn external_call_result(&mut self, call_id: u64, term: Option<Term>) -> PolarResult<()> {
        self.vm.external_call_result(call_id, term)
    }

    fn debug_command(&mut self, command: &str) -> PolarResult<()> {
        self.vm.debug_command(command)
    }

    fn clone_runnable(&self) -> Box<dyn Runnable> {
        Box::new(self.clone())
    }
}
