use std::cell::RefCell;
use std::collections::hash_map::{Entry, HashMap};
use std::rc::Rc;

use crate::counter::Counter;
use crate::error::PolarResult;
use crate::events::QueryEvent;
use crate::folder::{fold_value, Folder};
use crate::kb::Bindings;
use crate::partial::Partial;
use crate::runnable::Runnable;
use crate::terms::{Symbol, Term, Value};
use crate::visitor::{walk_term, Visitor};
use crate::vm::{Binding, BindingStack, Goal, Goals, PolarVirtualMachine};

#[derive(Clone)]
pub struct Inverter {
    vm: PolarVirtualMachine,
    bindings: Rc<RefCell<BindingStack>>,
    bsp: usize,
    results: Vec<BindingStack>,
    csps: HashMap<Symbol, usize>,
}

impl Inverter {
    pub fn new(
        vm: &PolarVirtualMachine,
        goals: Goals,
        bindings: Rc<RefCell<BindingStack>>,
        bsp: usize,
    ) -> Self {
        struct CspVisitor {
            csps: HashMap<Symbol, usize>,
        }

        impl Visitor for CspVisitor {
            fn visit_partial(&mut self, p: &Partial) {
                let v = p.variable.clone();
                let csp = p.constraints().len();
                if let Some(prev) = self.csps.insert(v.clone(), csp) {
                    assert_eq!(
                        prev, csp,
                        "csps don't match for {}\n\told: {}\n\tnew: {}",
                        v.0, prev, csp
                    );
                }
            }
        }

        let mut visitor = CspVisitor {
            csps: HashMap::new(),
        };

        goals.iter().for_each(|g| {
            if let Goal::Query { term } = g {
                walk_term(&mut visitor, &vm.deep_deref(term));
            }
        });

        Self {
            csps: visitor.csps,
            vm: vm.clone_with_goals(goals),
            bindings,
            bsp,
            results: vec![],
        }
    }
}

struct PartialInverter {
    csps: HashMap<Symbol, usize>,
}

impl PartialInverter {
    pub fn new(csps: HashMap<Symbol, usize>) -> Self {
        Self { csps }
    }

    fn invert_partial(&mut self, p: &Partial) -> Partial {
        let csp = self.csps.get(&p.variable).unwrap_or(&0);
        p.clone_with_constraints(p.inverted_constraints(*csp))
    }
}

impl Folder for PartialInverter {
    /// Invert top-level constraints.
    fn fold_term(&mut self, t: Term) -> Term {
        t.clone_with_value(match t.value() {
            Value::Partial(p) => Value::Partial(self.invert_partial(p)),
            v => fold_value(v.clone(), self),
        })
    }
}

/// Invert all partials in `bindings` and return them.
fn invert_partials(bindings: BindingStack, csps: HashMap<Symbol, usize>) -> BindingStack {
    let mut inverter = PartialInverter::new(csps);
    bindings
        .into_iter()
        .map(|Binding(var, value)| Binding(var, inverter.fold_term(value)))
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
    dedupe_bindings(bindings)
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
                        let csps = self.csps.clone();
                        self.bindings.borrow_mut().extend(
                            self.results
                                .drain(..)
                                .map(|b| invert_partials(b, csps.clone()))
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
                    if self.vm.query_contains_partial {
                        let bindings = self.vm.bindings[self.bsp..].to_owned();
                        let derefed = bindings
                            .into_iter()
                            .map(|Binding(var, value)| Binding(var, self.vm.deep_deref(&value)))
                            .collect();
                        self.results.push(derefed);
                    } else {
                        return Ok(QueryEvent::Done { result: false });
                    }
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
