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

struct PartialInverter<'a> {
    old_value: &'a Term,
}

impl<'a> PartialInverter<'a> {
    pub fn new(old_value: &'a Term) -> Self {
        Self { old_value }
    }

    fn invert_operation(&mut self, o: &Operation) -> Operation {
        // Compute csp from old_value vs. p.
        let csp = match self.old_value.value() {
            Value::Expression(e) => e.constraints().len(),
            _ => 0,
        };
        let p = o.clone_with_constraints(o.inverted_constraints(csp));
        eprintln!(
            "INVERTING w/old value {}: ¬{} = {}",
            self.old_value.to_polar(),
            o.clone().into_term().to_polar(),
            p.clone().into_term().to_polar()
        );
        p
    }
}

impl<'a> Folder for PartialInverter<'a> {
    /// Invert top-level constraints.
    fn fold_term(&mut self, t: Term) -> Term {
        t.clone_with_value(match t.value() {
            Value::Expression(o) => Value::Expression(self.invert_operation(o)),
            v => fold_value(v.clone(), self),
        })
    }
}

/// Invert all partials in `bindings` and return them.
fn invert_partials(bindings: BindingStack, old_bindings: &[Binding]) -> BindingStack {
    bindings
        .into_iter()
        .filter_map(|Binding(var, value)| {
            old_bindings
                .iter()
                .rfind(|Binding(v, _)| *v == var)
                .map(|Binding(_, old_value)| {
                    Binding(var, PartialInverter::new(old_value).fold_term(value))
                })
        })
        .collect()
}

/// Only keep latest bindings.
fn dedupe_bindings(bindings: BindingStack) -> Bindings {
    bindings
        .into_iter()
        .fold(Bindings::new(), |mut acc, Binding(var, value)| {
            acc.insert(var, value);
            acc
        })
}

/// Reduce + merge constraints.
fn reduce_constraints(mut acc: Bindings, bindings: BindingStack) -> Bindings {
    dedupe_bindings(bindings).drain().for_each(|(var, value)| {
        eprintln!("REDUCING {} = ({})", var, value);
        match acc.entry(var) {
            Entry::Occupied(mut o) => {
                let mut merged = o.get().value().as_expression().expect("expression").clone();
                let new = value.value().as_expression().expect("expression").clone();
                merged.merge_constraints(new);
                eprintln!("MERGED => {}", merged.to_polar());
                o.insert(value.clone_with_value(value!(merged)));
            }
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
                        let old_bindings = self.vm.bindings[..self.bsp].to_owned();
                        self.bindings.borrow_mut().extend(
                            self.results
                                .drain(..)
                                .map(|bindings| invert_partials(bindings, &old_bindings))
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
                    let derefed = bindings
                        .into_iter()
                        .map(|Binding(var, value)| Binding(var, self.vm.deep_deref(&value)))
                        .collect();
                    self.results.push(derefed);
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
