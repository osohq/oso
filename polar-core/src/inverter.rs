use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::bindings::{BindingManager, Bsp, FollowerId, VariableState};
use crate::counter::Counter;
use crate::error::{PolarError, PolarResult};
use crate::events::QueryEvent;
use crate::kb::Bindings;
use crate::partial::simplify_bindings;
use crate::runnable::Runnable;
use crate::terms::{Operation, Operator, Term, Value};
use crate::vm::{Goals, PolarVirtualMachine};

/// The inverter implements the `not` operation in Polar.
///
/// It is a `Runnable` that runs `goals` using `vm`, and returns inverted results.
///
/// If the inverter has no results, the inverted query is considered successful.
/// If the inverter has results, the new bindings or constraints made during the query
/// are inverted, and returned to the outer VM.
#[derive(Clone)]
pub struct Inverter {
    vm: PolarVirtualMachine,

    /// The bsp in VM when the inverter started.
    /// Used to determine which variables can have added constraints.
    bsp: Bsp,

    /// Acculumates new bindings from VM.
    results: Vec<BindingManager>,

    /// Constraints to return to parent VM.
    add_constraints: Rc<RefCell<Bindings>>,

    /// The ID of the current binding manager follower. Initialized in `run`.
    follower: Option<FollowerId>,

    /// An ID to distinguish logging from each inverter, useful when debugging
    /// queries with multiple nested `not` operations.
    _debug_id: u64,
}

static ID: AtomicU64 = AtomicU64::new(0);

impl Inverter {
    pub fn new(
        vm: &PolarVirtualMachine,
        goals: Goals,
        add_constraints: Rc<RefCell<Bindings>>,
        bsp: Bsp,
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

/// Convert a list of new bindings into inverted constraints.
///
/// `results` represents a disjunction not OR[result1, result2, ...].
///
/// To invert results, we:
///
/// 1. Invert each result.
/// 2. AND the inverted constraints together.
///
/// The output constraints are AND[!result1, !result2, ...].
fn results_to_constraints(results: Vec<BindingManager>) -> Bindings {
    let inverted = results.into_iter().map(invert_partials).collect();
    let reduced = reduce_constraints(inverted);
    let simplified = simplify_bindings(reduced).unwrap_or_default();

    simplified
        .into_iter()
        .map(|(k, v)| match v.value() {
            Value::Expression(_) => (k, v),
            _ => (
                k.clone(),
                v.clone_with_value(Value::Expression(op!(Unify, term!(k), v.clone()))),
            ),
        })
        .collect()
}

/// Invert constraints in `bindings`.
///
/// Constraints are inverted by getting each binding as a constraint.
/// Simplification is performed, to substitute bindings and remove temporary variables.
/// Then, each simplified expression is inverted.
/// A binding of `var` to `val` after simplification is converted into `var != val`.
fn invert_partials(bindings: BindingManager) -> Bindings {
    let mut new_bindings = Bindings::new();

    for var in bindings.variables() {
        let constraint = bindings.get_constraints(&var);
        new_bindings.insert(var.clone(), term!(constraint));
    }

    let simplified = simplify_bindings(new_bindings).unwrap_or_default();

    simplified
        .into_iter()
        .map(|(k, v)| match v.value() {
            Value::Expression(e) => (k, e.invert().into()),
            _ => (
                k.clone(),
                term!(op!(And, term!(op!(Neq, term!(k), v.clone())))),
            ),
        })
        .collect::<Bindings>()
}

/// Takes a vec of bindings and merges constraints on each variable.
fn reduce_constraints(bindings: Vec<Bindings>) -> Bindings {
    bindings
        .into_iter()
        .fold(Bindings::new(), |mut acc, bindings| {
            bindings
                .into_iter()
                .for_each(|(var, value)| match acc.entry(var.clone()) {
                    Entry::Occupied(mut o) => match (o.get().value(), value.value()) {
                        (Value::Expression(x), Value::Expression(y)) => {
                            let x = x.clone().merge_constraints(y.clone());
                            o.insert(value.clone_with_value(value!(x)));
                        }
                        (existing, new) => panic!(
                            "Illegal state reached while reducing constraints for {}: {} → {}",
                            var, existing, new
                        ),
                    },
                    Entry::Vacant(v) => {
                        v.insert(value);
                    }
                });
            acc
        })
}

/// Decide which variables come out of negation. This is hacky.
/// HACK: Remove known internal vars like `_value_*` and `_runnable_*`.
/// HACK: Do not emit constraints for variables that were unconstrained
/// (bound or unbound) before the inversion.
///
/// This prevents rules like `f(x) if not (w = 1) and x = w;` from working.
/// But, without this, an inverted query like:
/// f(x) if not g(x);
/// g(y) if y = 1;
///
/// ?= f(1);
///
/// incorrectly emits constraints on temporaries made when calling `g`,
/// like `_y_5`.
///
/// We can improve this by explicitly indicating to the simplifier
/// which variables are allowed.
fn filter_inverted_constraints(
    constraints: Bindings,
    vm: &PolarVirtualMachine,
    bsp: Bsp,
) -> Bindings {
    constraints
        .into_iter()
        .filter(|(k, _)| {
            !(matches!(
                vm.variable_state_at_point(k, &bsp),
                VariableState::Unbound | VariableState::Bound(_)
            ))
        })
        .collect::<Bindings>()
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
            // Binding followers are used to collect new bindings made during
            // the inversion.
            self.follower = Some(self.vm.add_binding_follower());
        }

        loop {
            // Pass most events through, but collect results and invert them.
            match self.vm.run(None)? {
                QueryEvent::Done { .. } => {
                    let result = self.results.is_empty();
                    if !result {
                        // If there are results, the inversion should usually fail. However,
                        // if those results have constraints we collect them and pass them
                        // out to the parent VM.
                        let constraints =
                            results_to_constraints(self.results.drain(..).collect::<Vec<_>>());
                        let mut bsp = Bsp::default();
                        // Use mem swap to avoid cloning bsps.
                        std::mem::swap(&mut self.bsp, &mut bsp);
                        let constraints = filter_inverted_constraints(constraints, &self.vm, bsp);

                        if !constraints.is_empty() {
                            // Return inverted constraints to parent VM.
                            // TODO (dhatch): Would be nice to come up with a better way of doing this.
                            self.add_constraints.borrow_mut().extend(constraints);

                            return Ok(QueryEvent::Done { result: true });
                        }
                    }
                    return Ok(QueryEvent::Done { result });
                }
                QueryEvent::Result { .. } => {
                    // Retrieve new bindings made when running inverted query.
                    let binding_follower = self
                        .vm
                        .remove_binding_follower(&self.follower.unwrap())
                        .unwrap();
                    self.results.push(binding_follower);
                    self.follower = Some(self.vm.add_binding_follower());
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

    fn handle_error(&mut self, error: PolarError) -> PolarResult<QueryEvent> {
        self.vm.handle_error(error)
    }
}
