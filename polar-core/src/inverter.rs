use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::bindings::{VariableState, BindingManager};
use crate::counter::Counter;
use crate::error::PolarResult;
use crate::events::QueryEvent;
use crate::formatting::ToPolarString;
use crate::kb::Bindings;
use crate::partial::simplify_bindings;
use crate::runnable::Runnable;
use crate::terms::{Operation, Operator, Term, Value};
use crate::vm::{Goals, PolarVirtualMachine};

#[derive(Clone)]
pub struct Inverter {
    vm: PolarVirtualMachine,
    bsp: usize,
    results: Vec<BindingManager>,
    add_constraints: Rc<RefCell<Bindings>>,
    follower: Option<usize>,
    _debug_id: u64,
}

static ID: AtomicU64 = AtomicU64::new(0);

impl Inverter {
    pub fn new(
        vm: &PolarVirtualMachine,
        goals: Goals,
        add_constraints: Rc<RefCell<Bindings>>,
        bsp: usize,
    ) -> Self {
        let mut vm = vm.clone_with_goals(goals);
        vm.inverting = true;
        Self {
            vm,
            bsp,
            add_constraints,
            results: vec![],
            follower: None,
            _debug_id: ID.fetch_add(1, Ordering::AcqRel),
        }
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
                        let constraints = results_to_constraints(self.results.drain(..).collect::<Vec<_>>());

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
                    let binding_follower = self.vm.remove_binding_follower(&self.follower.unwrap()).unwrap();
                    println!("{} removed follower {}", self._debug_id, self.follower.unwrap());
                    self.results.push(binding_follower);
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
