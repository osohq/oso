use std::collections::{hash_map::Entry, HashSet};

use crate::counter::Counter;
use crate::error::PolarResult;
use crate::events::QueryEvent;
use crate::folder::Folder;
use crate::kb::Bindings;
use crate::partial::Constraints;
use crate::runnable::Runnable;
use crate::terms::{Operation, Operator, Symbol, Term, Value};
use crate::vm::{Binding, BindingStack, Goals, PolarVirtualMachine};

#[derive(Clone)]
pub struct Inverter {
    vm: PolarVirtualMachine,
    results: Vec<BindingStack>,
}

impl Inverter {
    pub fn new(vm: &PolarVirtualMachine, goals: Goals) -> Self {
        Self {
            vm: vm.clone_with_bindings(goals),
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
    fn fold_operation(&mut self, o: Operation) -> Operation {
        Operation {
            operator: match o.operator {
                Operator::And => Operator::Or,
                Operator::Or => Operator::And,
                Operator::Unify | Operator::Eq => Operator::Neq,
                Operator::Neq => Operator::Unify,
                Operator::Gt => Operator::Leq,
                Operator::Geq => Operator::Lt,
                Operator::Lt => Operator::Geq,
                Operator::Leq => Operator::Gt,
                _ => todo!("negate {:?}", o.operator),
            },
            args: self.fold_list(o.args),
        }
    }

    // If there are any constraints to invert, invert 'em.
    fn fold_constraints(&mut self, c: Constraints) -> Constraints {
        if !c.operations.is_empty() {
            let new_binding = Binding(
                c.variable.clone(),
                Term::new_temporary(Value::Partial(Constraints {
                    variable: c.variable.clone(),
                    operations: vec![Operation {
                        operator: Operator::Or,
                        args: c
                            .operations
                            .iter()
                            .cloned()
                            .map(|o| Term::new_temporary(Value::Expression(self.fold_operation(o))))
                            .collect(),
                    }],
                })),
            );
            self.new_bindings.push(new_binding);
        }
        c
    }
}

/// If there are no partials, and you get no results, return true
/// If there are no partials, and you get at least one result, return false
/// If there's a partial, return `true` with the partial.
///     - what if the partial has no operations?
impl Runnable for Inverter {
    fn run(
        &mut self,
        bindings: Option<&mut BindingStack>,
        bsp: Option<&mut usize>,
        _: Option<&mut Counter>,
    ) -> PolarResult<QueryEvent> {
        let bsp = bsp.expect("Inverter needs a BSP");
        loop {
            // Pass most events through, but collect results and invert them.
            if let Ok(event) = self.vm.run(None, None, None) {
                match event {
                    QueryEvent::Done { .. } => {
                        let mut result = self.results.is_empty();
                        if !result {
                            let new_bindings: BindingStack = self
                                .results
                                .iter()
                                .map(|result| {
                                    let mut inverter = ConstraintInverter::new();
                                    result.iter().for_each(|Binding(_, value)| {
                                        inverter.fold_term(value.clone());
                                    });
                                    inverter.new_bindings
                                })
                                .fold(Bindings::new(), |mut acc, bindings| {
                                    let mut seen = HashSet::<Symbol>::new();
                                    for Binding(var, value) in bindings.into_iter().rev() {
                                        if seen.contains(&var) {
                                            continue;
                                        } else {
                                            seen.insert(var.clone());
                                        }

                                        match acc.entry(var) {
                                            Entry::Occupied(mut o) => {
                                                let existing = o.get();
                                                if let Value::Partial(existing) = existing.value() {
                                                    if let Ok(new) = value.value().as_partial() {
                                                        assert_eq!(existing.variable, new.variable);
                                                        let conjunction = value.clone_with_value(
                                                            Value::Partial(Constraints {
                                                                variable: existing.variable.clone(),
                                                                operations: existing
                                                                    .operations
                                                                    .iter()
                                                                    .cloned()
                                                                    .chain(
                                                                        new.operations
                                                                            .iter()
                                                                            .cloned(),
                                                                    )
                                                                    .collect(),
                                                            }),
                                                        );
                                                        o.insert(conjunction);
                                                        break;
                                                    } else {
                                                        unreachable!();
                                                    }
                                                } else {
                                                    unreachable!();
                                                }
                                            }
                                            Entry::Vacant(v) => {
                                                v.insert(value);
                                            }
                                        }
                                    }
                                    acc
                                })
                                .drain()
                                .map(|(var, value)| Binding(var, value))
                                .collect();

                            if let Some(bindings) = bindings {
                                if !new_bindings.is_empty() {
                                    *bsp += new_bindings.len();
                                    bindings.extend(new_bindings);
                                    result = true;
                                }
                            }
                        }
                        return Ok(QueryEvent::Done { result });
                    }
                    QueryEvent::Result { .. } => {
                        let bindings = self.vm.bindings[*bsp..].to_owned();
                        self.results.push(bindings);
                    }
                    event => return Ok(event),
                }
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
