use std::cell::RefCell;
use std::collections::hash_map::{Entry, HashMap};
use std::rc::Rc;

use crate::counter::Counter;
use crate::error::PolarResult;
use crate::events::QueryEvent;
use crate::folder::{fold_value, Folder};
use crate::kb::Bindings;
use crate::partial::Constraints;
use crate::runnable::Runnable;
use crate::terms::{Symbol, Term, Value};
use crate::visitor::{walk_term, Visitor};
use crate::vm::{Binding, BindingStack, Goal, Goals, PolarVirtualMachine};

#[derive(Clone)]
pub struct Inverter {
    vm: PolarVirtualMachine,
    bindings: Rc<RefCell<BindingStack>>,
    initial_partials: Bindings,
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
            fn visit_constraints(&mut self, c: &Constraints) {
                let v = c.variable.clone();
                let csp = c.operations().len();
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

        let mut vm = vm.clone_with_goals(goals);
        // Remove partials from vm bindings.
        let (partials, no_partials) = partition_partials(vm.bindings.clone());

        // Rebind partials to empty partial (cannot remove from bindings because that invalidates all pointers.)
        for Binding(var, partial) in partials.iter() {
            if partial.value().as_partial().unwrap().operations.is_empty() {
                // Don't rebind partials that are already empty.
                continue;
            }

            vm.bindings.push(Binding(var.clone(), partial.clone_with_value(Value::Partial(Constraints::new(var.clone())))));
        }

        let bsp = vm.bindings.len();

        Self {
            csps: visitor.csps,
            initial_partials: reduce_constraints(Bindings::new(), partials),
            vm,
            bindings,
            bsp,
            results: vec![],
        }
    }
}

struct ConstraintInverter {
    pub new_bindings: BindingStack,
    csps: HashMap<Symbol, usize>,
}

impl ConstraintInverter {
    pub fn new(csps: HashMap<Symbol, usize>) -> Self {
        Self {
            csps,
            new_bindings: vec![],
        }
    }

    fn invert_constraints(&mut self, c: &Constraints) -> Constraints {
        let csp = 0;
        let partial = c.clone_with_operations(c.inverted_operations(csp));
        self.new_bindings.push(Binding(
            partial.variable.clone(),
            Term::new_temporary(Value::Partial(partial.clone())),
        ));
        partial
    }
}

impl Folder for ConstraintInverter {
    /// Invert top-level constraints.
    fn fold_term(&mut self, t: Term) -> Term {
        t.clone_with_value(match t.value() {
            Value::Partial(c) => Value::Partial(self.invert_constraints(c)),
            v => fold_value(v.clone(), self),
        })
    }
}

/// Invert constraints on all partials in `bindings` and return them.
fn invert_constraints(bindings: BindingStack, csps: HashMap<Symbol, usize>) -> BindingStack {
    let mut inverter = ConstraintInverter::new(csps);
    for Binding(_, value) in bindings.iter() {
        inverter.fold_term(value.clone());
    }
    inverter.new_bindings
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

/// Remove partials from bindings.
fn partition_partials(bindings: BindingStack) -> (BindingStack, BindingStack) {
    bindings.into_iter()
        .partition(|Binding(var, term)| matches!(term.value(), Value::Partial(_)))
}

/// Reduce + merge constraints.
fn reduce_constraints(mut acc: Bindings, bindings: BindingStack) -> Bindings {
    dedupe_bindings(bindings)
        .drain()
        .for_each(|(var, value)| match acc.entry(var) {
            Entry::Occupied(mut o) => {
                // TODO(gj): Does this ever get hit?
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
                                .map(|b| invert_constraints(b, csps.clone()))
                                .fold(self.initial_partials.clone(), reduce_constraints)
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
