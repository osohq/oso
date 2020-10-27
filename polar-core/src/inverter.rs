//! A Runnable that runs a query and inverts the results in three ways:
//!
//! 1. If no results are emitted (indicating failure), return true.
//! 2. If at least one result is emitted containing a partial, invert the partial's constraints,
//!    pass the inverted partials back to the parent Runnable via a shared BindingStack, and return
//!    true.
//! 3. In all other cases, return false.

use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::rc::Rc;

use crate::counter::Counter;
use crate::error::PolarResult;
use crate::events::QueryEvent;
use crate::folder::{fold_operation, Folder};
use crate::kb::Bindings;
use crate::partial::Constraints;
use crate::runnable::Runnable;
use crate::terms::{Operation, Operator, Term, Value};
use crate::vm::{Binding, BindingStack, Goals, PolarVirtualMachine};

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
        Self {
            vm: vm.clone_with_bindings(goals),
            bindings,
            bsp,
            results: vec![],
        }
    }
}

struct ConstraintInverter {
    pub new_bindings: BindingStack,
}

impl ConstraintInverter {
    pub fn new() -> Self {
        Self {
            new_bindings: vec![],
        }
    }
}

impl Folder for ConstraintInverter {
    /// Invert operators.
    fn fold_operator(&mut self, o: Operator) -> Operator {
        match o {
            Operator::And => Operator::Or,
            Operator::Or => Operator::And,
            Operator::Unify | Operator::Eq => Operator::Neq,
            Operator::Neq => Operator::Unify,
            Operator::Gt => Operator::Leq,
            Operator::Geq => Operator::Lt,
            Operator::Lt => Operator::Geq,
            Operator::Leq => Operator::Gt,
            _ => todo!("negate {:?}", o),
        }
    }

    /// Invert constraints.
    fn fold_constraints(&mut self, c: Constraints) -> Constraints {
        let partial = match c.operations.len() {
            // Do nothing to an empty partial.
            0 => return c,

            // Invert a single constraint.
            1 => Constraints {
                variable: c.variable.clone(),
                operations: vec![fold_operation(c.operations[0].clone(), self)],
            },

            // Invert the conjunction of multiple constraints, yielding a disjunction of their
            // inverted selves. (De Morgan's Law)
            _ => Constraints {
                variable: c.variable.clone(),
                operations: vec![Operation {
                    operator: Operator::Or,
                    args: c
                        .operations
                        .into_iter()
                        .map(|o| Term::new_temporary(Value::Expression(fold_operation(o, self))))
                        .collect(),
                }],
            },
        };
        self.new_bindings.push(Binding(
            partial.variable.clone(),
            Term::new_temporary(Value::Partial(partial.clone())),
        ));
        partial
    }
}

/// Invert constraints on all partials in `bindings` and return them.
fn invert_constraints(bindings: BindingStack) -> BindingStack {
    let mut inverter = ConstraintInverter::new();
    for Binding(_, value) in bindings.iter() {
        inverter.fold_term(value.clone());
    }
    inverter.new_bindings
}

/// Only keep latest bindings.
fn reduce_bindings(bindings: BindingStack) -> Bindings {
    bindings
        .into_iter()
        .fold(Bindings::new(), |mut acc, Binding(var, value)| {
            acc.insert(var, value);
            acc
        })
}

/// Reduce + merge constraints.
fn reduce_constraints(mut acc: Bindings, bindings: BindingStack) -> Bindings {
    reduce_bindings(bindings)
        .drain()
        .for_each(|(var, value)| match acc.entry(var) {
            Entry::Occupied(mut o) => {
                let mut old = o.get().value().as_partial().expect("Partial").clone();
                let new = value.value().as_partial().expect("Partial").clone();
                old.merge_constraints(new);
                let conjunction = value.clone_with_value(Value::Partial(old));
                o.insert(conjunction);
            }
            Entry::Vacant(v) => {
                v.insert(value);
            }
        });
    acc
}

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
                                .map(invert_constraints)
                                .fold(Bindings::new(), reduce_constraints)
                                .drain()
                                .map(|(var, value)| {
                                    // We have at least one partial to return, so succeed.
                                    result = true;

                                    Binding(var, value)
                                }),
                        );
                    }
                    return Ok(QueryEvent::Done { result });
                }
                QueryEvent::Result { .. } => {
                    let bindings = self.vm.bindings[self.bsp..].to_owned();
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

    fn clone_runnable(&self) -> Box<dyn Runnable> {
        Box::new(self.clone())
    }
}
