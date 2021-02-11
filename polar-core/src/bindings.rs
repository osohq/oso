/// Manage binding state in the VM.
///
/// Bindings associate variables in the VM with constraints or values.
use std::collections::{HashMap, HashSet};

use crate::error::PolarResult;
use crate::folder::{fold_term, Folder};
use crate::formatting::ToPolarString;
use crate::terms::{has_rest_var, Operation, Operator, Symbol, Term, Value};
use crate::vm::cycle_constraints;

#[derive(Clone, Debug)]
pub struct Binding(pub Symbol, pub Term);

// TODO This is only public for debugger and inverter.
// Eventually this should be an internal interface.
pub type BindingStack = Vec<Binding>;
pub type Bindings = HashMap<Symbol, Term>;

pub type Bsp = usize;

/// Variable binding state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VariableState {
    Unbound,
    Bound(Term),
    Cycle(Vec<Symbol>),
    Partial(Operation),
}

#[derive(Clone, Debug, Default)]
/// The binding manager is responsible for managing binding & constraint state.
/// It is updated primarily using:
/// - `bind`
/// - `add_constraint`
///
/// Bindings are retrived with:
/// - `deref`
/// - `value`
/// - `variable_state`
/// - `bindings`
pub struct BindingManager {
    bindings: BindingStack,
}

impl BindingManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Bind `var` to `val`.
    pub fn bind(&mut self, var: &Symbol, val: Term) {
        self.bindings.push(Binding(var.clone(), val));
    }

    /// Look up a variable in the bindings stack and return
    /// a reference to its value if it's bound.
    pub fn value(&self, variable: &Symbol, bsp: usize) -> Option<&Term> {
        self.bindings[..bsp]
            .iter()
            .rev()
            .find(|Binding(var, _)| var == variable)
            .map(|Binding(_, val)| val)
    }

    /// If `term` is a variable, return the value bound to that variable.
    /// If `term` is a list, dereference all items in the list.
    /// Otherwise, return `term`.
    pub fn deref(&self, term: &Term) -> Term {
        match &term.value() {
            Value::List(list) => {
                // Deref all elements.
                let mut derefed: Vec<Term> =
                    // TODO(gj): reduce recursion here.
                    list.iter().map(|t| self.deref(t)).collect();

                // If last element was a rest variable, append the list it derefed to.
                if has_rest_var(list) {
                    if let Some(last_term) = derefed.pop() {
                        if let Value::List(terms) = last_term.value() {
                            derefed.append(&mut terms.clone());
                        } else {
                            derefed.push(last_term);
                        }
                    }
                }

                term.clone_with_value(Value::List(derefed))
            }
            Value::Variable(v) => match self.variable_state(v) {
                VariableState::Bound(value) => value,
                _ => term.clone(),
            },
            Value::RestVariable(v) => match self.variable_state(v) {
                VariableState::Bound(value) => match value.value() {
                    Value::List(l) if has_rest_var(l) => self.deref(&value),
                    _ => value,
                },
                _ => term.clone(),
            },
            _ => term.clone(),
        }
    }

    /// Dereference all variables in term, including within nested structures like
    /// lists and dictionaries.
    /// Do not dereference variables inside expressions.
    pub fn deep_deref(&self, term: &Term) -> Term {
        pub struct Derefer<'a> {
            binding_manager: &'a BindingManager,
        }

        impl<'a> Derefer<'a> {
            pub fn new(binding_manager: &'a BindingManager) -> Self {
                Self { binding_manager }
            }
        }

        impl<'a> Folder for Derefer<'a> {
            fn fold_term(&mut self, t: Term) -> Term {
                match t.value() {
                    Value::List(_) => fold_term(self.binding_manager.deref(&t), self),
                    Value::Variable(_) | Value::RestVariable(_) => {
                        let derefed = self.binding_manager.deref(&t);
                        match derefed.value() {
                            Value::Expression(_) => t,
                            _ => fold_term(derefed, self),
                        }
                    }
                    _ => fold_term(t, self),
                }
            }
        }

        Derefer::new(self).fold_term(term.clone())
    }

    /// Check the state of `variable`.
    pub fn variable_state(&self, variable: &Symbol) -> VariableState {
        self.variable_state_at_point(variable, self.bsp())
    }

    // TODO: Get rid of this, only used in inverter.
    /// Check the state of `variable` at `bsp`.
    pub fn variable_state_at_point(&self, variable: &Symbol, bsp: Bsp) -> VariableState {
        let mut path = vec![variable];
        while let Some(value) = self.value(path.last().unwrap(), bsp) {
            match value.value() {
                Value::Expression(e) => return VariableState::Partial(e.clone()),
                Value::Variable(v) | Value::RestVariable(v) => {
                    if v == variable {
                        return VariableState::Cycle(path.into_iter().cloned().collect());
                    } else {
                        path.push(v);
                    }
                }
                _ => return VariableState::Bound(value.clone()),
            }
        }
        VariableState::Unbound
    }

    /// Add `term` as a constraint.
    pub fn add_constraint(&mut self, term: &Term) -> PolarResult<()> {
        let Operation { operator: op, args } = term.value().as_expression().unwrap();
        assert!(
            !matches!(*op, Operator::And | Operator::Or),
            "Expected a bare constraint."
        );
        assert!(args.len() >= 2);

        let (left, right) = (&args[0], &args[1]);
        match (left.value(), right.value()) {
            (Value::Variable(left_name), Value::Variable(right_name)) => {
                match (
                    self.variable_state(left_name),
                    self.variable_state(right_name),
                ) {
                    (VariableState::Unbound, VariableState::Unbound) => {
                        self.constrain(&op!(And, term.clone()))?;
                    }
                    (VariableState::Cycle(left_cycle), VariableState::Cycle(right_cycle)) => {
                        let mut merged_cycles = cycle_constraints(left_cycle);
                        merged_cycles.merge_constraints(cycle_constraints(right_cycle));
                        self.constrain(&merged_cycles.clone_with_new_constraint(term.clone()))?;
                    }
                    (VariableState::Partial(partial), VariableState::Unbound)
                    | (VariableState::Unbound, VariableState::Partial(partial)) => {
                        self.constrain(&partial.clone_with_new_constraint(term.clone()))?;
                    }
                    (
                        VariableState::Partial(mut left_partial),
                        VariableState::Partial(right_partial),
                    ) => {
                        left_partial.merge_constraints(right_partial);
                        self.constrain(&left_partial.clone_with_new_constraint(term.clone()))?;
                    }
                    (VariableState::Partial(mut partial), VariableState::Cycle(cycle))
                    | (VariableState::Cycle(cycle), VariableState::Partial(mut partial)) => {
                        partial.merge_constraints(cycle_constraints(cycle));
                        self.constrain(&partial.clone_with_new_constraint(term.clone()))?;
                    }
                    (VariableState::Cycle(cycle), VariableState::Unbound)
                    | (VariableState::Unbound, VariableState::Cycle(cycle)) => {
                        let partial = cycle_constraints(cycle);
                        self.constrain(&partial.clone_with_new_constraint(term.clone()))?;
                    }
                    (VariableState::Bound(left_value), _) => {
                        panic!(
                            "Variable {} unexpectedly bound to {} in constraint {}.",
                            left.to_polar(),
                            left_value.to_polar(),
                            term.to_polar(),
                        );
                    }
                    (_, VariableState::Bound(right_value)) => {
                        panic!(
                            "Variable {} unexpectedly bound to {} in constraint {}.",
                            right.to_polar(),
                            right_value.to_polar(),
                            term.to_polar(),
                        );
                    }
                }
            }
            (Value::Variable(name), _) | (_, Value::Variable(name)) => {
                match self.variable_state(name) {
                    VariableState::Unbound => {
                        self.constrain(&op!(And, term.clone()))?;
                    }
                    VariableState::Cycle(cycle) => {
                        let partial = cycle_constraints(cycle);
                        self.constrain(&partial.clone_with_new_constraint(term.clone()))?;
                    }
                    VariableState::Partial(partial) => {
                        self.constrain(&partial.clone_with_new_constraint(term.clone()))?;
                    }
                    VariableState::Bound(value) => {
                        panic!(
                            "Variable {} unexpectedly bound to {} in constraint {}.",
                            name.0,
                            value.to_polar(),
                            term.to_polar()
                        );
                    }
                }
            }
            (_, _) => panic!("At least one side of a constraint expression must be a variable."),
        }

        Ok(())
    }

    // TODO: non public, the only way to add constraints should be `add_constraint`.
    pub fn constrain(&mut self, o: &Operation) -> PolarResult<()> {
        assert_eq!(o.operator, Operator::And, "bad constraint {}", o.to_polar());
        for var in o.variables() {
            match self.variable_state(&var) {
                VariableState::Bound(_) => (),
                _ => self.bind(&var, o.clone().into_term()),
            }
        }
        Ok(())
    }

    /// Reset the state of `BindingManager` to what it was at `to`.
    pub fn backtrack(&mut self, to: Bsp) {
        self.bindings.truncate(to)
    }

    /// Retrieve an opaque value representing the current state of `BindingManager`.
    /// Can be used to reset state with `backtrack`.
    pub fn bsp(&self) -> Bsp {
        self.bindings.len()
    }

    pub fn bindings(&self, include_temps: bool) -> Bindings {
        self.bindings_after(include_temps, 0)
    }

    pub fn bindings_after(&self, include_temps: bool, after: Bsp) -> Bindings {
        let mut bindings = HashMap::new();
        for Binding(var, value) in &self.bindings[after..] {
            if !include_temps && var.is_temporary_var() {
                continue;
            }
            bindings.insert(var.clone(), self.deep_deref(value));
        }
        bindings
    }

    // TODO rename to deep_deref_batch
    pub fn variable_bindings(&self, variables: &HashSet<Symbol>) -> Bindings {
        let mut bindings = HashMap::new();
        for var in variables.iter() {
            let value = self.value(var, self.bsp());
            if let Some(value) = value {
                bindings.insert(var.clone(), self.deep_deref(value));
            }
        }
        bindings
    }

    /// Get the bindings stack *for debugging purposes only*.
    pub fn bindings_debug(&self) -> &BindingStack {
        &self.bindings
    }

    // TODO maybe port from VM:
    // relevant_bindings
    // variable_bindings
    // bindings
}
