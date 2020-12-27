// TODO(gj): fix term!
use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::rc::Rc;

use crate::counter::Counter;
use crate::error::PolarResult;
use crate::events::QueryEvent;
use crate::folder::{fold_value, Folder};
use crate::formatting::ToPolarString;
use crate::kb::Bindings;
use crate::runnable::Runnable;
use crate::terms::{Operation, Operator, Symbol, Term, Value};
use crate::vm::{
    cycle_constraints, Binding, BindingStack, Goals, PolarVirtualMachine, VariableState,
};

#[derive(Clone)]
pub struct Inverter {
    vm: PolarVirtualMachine,
    bindings: Rc<RefCell<BindingStack>>,
    bsp: usize,
    results: Vec<BindingStack>,
}

impl Inverter {
    pub fn new(
        vm: &PolarVirtualMachine,
        goals: Goals,
        bindings: Rc<RefCell<BindingStack>>,
        bsp: usize,
    ) -> Self {
        let mut vm = vm.clone_with_goals(goals);
        vm.simplify = false;
        Self {
            vm,
            bindings,
            bsp,
            results: vec![],
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
        let p = o.clone_with_constraints(o.inverted_constraints(csp));
        eprintln!(
            "INVERTING w/old state {:?}: ¬{} = {}",
            self.old_state,
            o.clone().into_term().to_polar(),
            p.clone().into_term().to_polar()
        );
        p
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

/// Invert partial values in `bindings` with respect to the old VM bindings.
fn invert_partials(bindings: BindingStack, vm: &PolarVirtualMachine) -> BindingStack {
    let mut new_bindings = Vec::new();
    for Binding(var, value) in bindings {
        match vm.variable_state(&var) {
            VariableState::Unbound => (),
            VariableState::Bound(x) => {
                todo!("{} is bound to {} in VM, now {}", var, x, value.to_polar())
            }
            VariableState::Cycle(c) => {
                let mut constraints = cycle_constraints(c.clone());
                constraints.merge_constraints(match value.value() {
                    Value::Expression(e) => e.clone(),
                    _ => op!(And, term!(op!(Unify, term!(var.clone()), value.clone()))),
                });
                for var in constraints.variables() {
                    new_bindings.push(Binding(
                        var.clone(),
                        PartialInverter::new(var.clone(), VariableState::Cycle(c.clone()))
                            .fold_term(constraints.clone().into_term()),
                    ));
                }
            }
            VariableState::Partial(e) => todo!(
                "{} was partial {} in VM, now {}",
                var,
                e.to_polar(),
                value.to_polar()
            ),
        }
    }
    new_bindings
}

/// Only keep latest bindings.
fn dedupe_bindings(bindings: BindingStack) -> Bindings {
    bindings
        .into_iter()
        .inspect(|Binding(var, val)| {
            eprintln!("DEDUPING: {} -> {}", var, val.to_polar());
        })
        .fold(Bindings::new(), |mut acc, Binding(var, value)| {
            acc.insert(var, value);
            acc
        })
}

/// Reduce + merge constraints.
fn reduce_constraints(mut acc: Bindings, bindings: BindingStack) -> Bindings {
    dedupe_bindings(bindings).drain().for_each(|(var, value)| {
        eprintln!("REDUCING {} = ({})", var, value);
        match acc.entry(var.clone()) {
            Entry::Occupied(mut o) => match (o.get().value(), value.value()) {
                (Value::Expression(x), Value::Expression(y)) => {
                    let mut x = x.clone();
                    x.merge_constraints(y.clone());
                    eprintln!("MERGED => {}", x.to_polar());
                    o.insert(value.clone_with_value(value!(x)));
                }
                (Value::Expression(x), _) => {
                    let unify = op!(Unify, term!(var), value.clone());
                    eprintln!(
                        "CLONED WITH NEW CONSTRAINT {}; {}",
                        x.to_polar(),
                        unify.to_polar()
                    );
                    let x = x.clone_with_new_constraint(term!(unify));
                    o.insert(value.clone_with_value(value!(x)));
                }
                (_, Value::Expression(x)) => {
                    let unify = op!(Unify, term!(var), o.get().clone());
                    eprintln!(
                        "CLONED WITH NEW CONSTRAINT {}; {}",
                        x.to_polar(),
                        unify.to_polar()
                    );
                    let x = x.clone_with_new_constraint(term!(unify));
                    o.insert(value.clone_with_value(value!(x)));
                }
                _ => {
                    let left_unify = term!(value!(op!(Unify, term!(var.clone()), o.get().clone())));
                    let right_unify = term!(value!(op!(Unify, term!(var), value.clone())));
                    let x = op!(And, left_unify, right_unify);
                    eprintln!("CREATED NEW PARTIAL {}", x.to_polar());
                    o.insert(value.clone_with_value(value!(x)));
                }
            },
            Entry::Vacant(v) => {
                v.insert(value);
            }
        }
    });
    acc
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
        loop {
            // Pass most events through, but collect results and invert them.
            match self.vm.run(None)? {
                QueryEvent::Done { .. } => {
                    let mut result = self.results.is_empty();
                    if !result {
                        self.bindings.borrow_mut().extend(
                            self.results
                                .drain(..)
                                .collect::<Vec<BindingStack>>()
                                .into_iter()
                                .map(|bindings| invert_partials(bindings, &self.vm))
                                .into_iter()
                                .fold(Bindings::new(), reduce_constraints) // loses ordering
                                .into_iter()
                                .map(|(var, value)| {
                                    // We have at least one binding to return, so succeed.
                                    result = true;

                                    Binding(var, value)
                                }),
                        );
                    }
                    return Ok(QueryEvent::Done { result });
                }
                QueryEvent::Result { .. } => {
                    let bindings: BindingStack = self.vm.bindings.drain(self.bsp..).collect();
                    self.results.push(bindings);
                }
                event => return Ok(event),
            }
        }
    }

    fn external_question_result(&mut self, call_id: u64, answer: bool) -> PolarResult<()> {
        self.vm.external_question_result(call_id, answer)
    }

    fn external_error(&mut self, message: String) -> PolarResult<()> {
        self.vm.external_error(message)
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
