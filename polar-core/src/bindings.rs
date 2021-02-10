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

// TODO only public for debugger.. how can we handle this
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

#[derive(Clone, Debug)]
pub struct BindingManager {
    bindings: BindingStack,
}

impl BindingManager {
    pub fn new() -> Self {
        Self { bindings: vec![] }
    }

    /// Bind `var` to `val`, overwriting any already bound value.
    pub fn rebind(&mut self, var: &Symbol, val: Term) {
        self.add_binding(var, val);
    }

    /// Bind `var` to `val`.
    ///
    ///
    /// If `var` is already bound or constrained, the
    /// binding or constraints are replaced with `val`.
    pub fn bind(&mut self, var: &Symbol, val: Term) {
        if let Ok(symbol) = val.value().as_symbol() {
            self.bind_variables(var, symbol);
        } else {
            self.add_binding(var, val);
        }
    }

    /// Bind two variables together.
    fn bind_variables(&mut self, x: &Symbol, y: &Symbol) {
        match (self.variable_state(x), self.variable_state(y)) {
            (VariableState::Bound(_), VariableState::Unbound) => {
                // Replace binding.
                self.add_binding(x, term!(y.clone()));
            },
            (VariableState::Unbound, VariableState::Bound(_)) => {
                // Bind variables in cycle.
                if x != y {
                    self.add_binding(x, term!(y.clone()));
                    self.add_binding(y, term!(x.clone()));
                }
            }

            // Cycles: one or more variables are bound together.
            (VariableState::Unbound, VariableState::Unbound) => {
                // Both variables are unbound. Bind them in a new cycle,
                // but do not create 1-cycles.
                if x != y {
                    self.add_binding(x, term!(y.clone()));
                    self.add_binding(y, term!(x.clone()));
                }
            }
            (VariableState::Cycle(c), VariableState::Unbound) => {
                // Left is in a cycle. Extend it to include right.
                let p = c.last().unwrap();
                assert_ne!(p, x);
                self.add_binding(p, term!(y.clone()));
                self.add_binding(y, term!(x.clone()));
            }
            (VariableState::Unbound, VariableState::Cycle(d)) => {
                // Right is in a cycle. Extend it to include left.
                let q = d.last().unwrap();
                assert_ne!(q, y);
                self.add_binding(q, term!(x.clone()));
                self.add_binding(x, term!(y.clone()));
            }
            (VariableState::Cycle(c), VariableState::Cycle(d)) => {
                // Both variables are in cycles.
                let h = c.iter().collect::<HashSet<&Symbol>>();
                let i = d.iter().collect::<HashSet<&Symbol>>();
                if h.intersection(&i).next().is_some() {
                    // The cycles must be the same. Do nothing.
                    assert_eq!(h, i);
                } else {
                    // Join the two cycles.
                    let p = c.last().unwrap();
                    let q = d.last().unwrap();
                    assert_ne!(p, x);
                    assert_ne!(q, y);
                    self.add_binding(p, term!(y.clone()));
                    self.add_binding(q, term!(x.clone()));
                }
            }
            (VariableState::Cycle(_), VariableState::Bound(y)) => {
                // Ground out the cycle.
                self.add_binding(x, y);
            }
            (VariableState::Bound(_), VariableState::Cycle(d)) => {
                // Left is currently bound. Instead, rebind it by adding it to
                // the cycle.
                let q = d.last().unwrap();
                assert_ne!(q, y);
                self.add_binding(q, term!(x.clone()));
                self.add_binding(x, term!(y.clone()));
            },
            (VariableState::Bound(_), VariableState::Bound(y)) => {
                self.add_binding(x, y);
            },
            (VariableState::Bound(l), VariableState::Partial(_)) => {
                // Left is bound, right has constraints.
                // TODO (dhatch): No unwrap.
                self.add_constraint(&op!(Unify, l.clone(), term!(y.clone())).into_term()).unwrap();
            },
            (VariableState::Partial(_), VariableState::Bound(right)) => {
                self.add_constraint(&op!(Unify, term!(x.clone()), right).into_term()).unwrap();
            },
            (VariableState::Partial(_), _) | (_, VariableState::Partial(_)) => {
                self.add_constraint(&op!(Unify, term!(x.clone()), term!(y.clone())).into_term()).unwrap();
            },
        }
    }

    fn add_binding(&mut self, var: &Symbol, val: Term) {
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

    pub fn variable_state(&self, variable: &Symbol) -> VariableState {
        self.variable_state_at_point(variable, self.bsp())
    }

    // TODO: get rid of this.
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

    pub fn add_constraint(&mut self, term: &Term) -> PolarResult<()> {
        let Operation { operator: op, args } = term.value().as_expression().unwrap();
        assert!(
            !matches!(*op, Operator::And | Operator::Or),
            "Expected a bare constraint."
        );
        assert!(args.len() >= 2);

        let (left, right) = (&args[0], &args[1]);
        match (
            extract_variable(left.value()),
            extract_variable(right.value()),
        ) {
            (Value::Variable(l), Value::Variable(r)) => {
                match (self.variable_state(l), self.variable_state(r)) {
                    (VariableState::Unbound, VariableState::Unbound) => {
                        self.constrain(&op!(And, term.clone()))?;
                    }
                    (VariableState::Cycle(c), VariableState::Cycle(d)) => {
                        let mut e = cycle_constraints(c);
                        e.merge_constraints(cycle_constraints(d));
                        self.constrain(&e.clone_with_new_constraint(term.clone()))?;
                    }
                    (VariableState::Partial(e), VariableState::Unbound)
                    | (VariableState::Unbound, VariableState::Partial(e)) => {
                        self.constrain(&e.clone_with_new_constraint(term.clone()))?;
                    }
                    (VariableState::Partial(mut e), VariableState::Partial(f)) => {
                        e.merge_constraints(f);
                        self.constrain(&e.clone_with_new_constraint(term.clone()))?;
                    }
                    (VariableState::Partial(mut e), VariableState::Cycle(c))
                    | (VariableState::Cycle(c), VariableState::Partial(mut e)) => {
                        e.merge_constraints(cycle_constraints(c));
                        self.constrain(&e.clone_with_new_constraint(term.clone()))?;
                    }
                    (VariableState::Cycle(c), VariableState::Unbound)
                    | (VariableState::Unbound, VariableState::Cycle(c)) => {
                        let e = cycle_constraints(c);
                        self.constrain(&e.clone_with_new_constraint(term.clone()))?;
                    }
                    (VariableState::Bound(x), _) => {
                        panic!(
                            "Variable {} unexpectedly bound to {} in constraint {}.",
                            left.to_polar(),
                            x.to_polar(),
                            term.to_polar(),
                        );
                    }
                    (_, VariableState::Bound(x)) => {
                        panic!(
                            "Variable {} unexpectedly bound to {} in constraint {}.",
                            right.to_polar(),
                            x.to_polar(),
                            term.to_polar(),
                        );
                    }
                }
            }
            (Value::Variable(v), _) | (_, Value::Variable(v)) => match self.variable_state(v) {
                VariableState::Unbound => {
                    self.constrain(&op!(And, term.clone()))?;
                }
                VariableState::Cycle(c) => {
                    let e = cycle_constraints(c);
                    self.constrain(&e.clone_with_new_constraint(term.clone()))?;
                }
                VariableState::Partial(e) => {
                    self.constrain(&e.clone_with_new_constraint(term.clone()))?;
                }
                VariableState::Bound(x) => {
                    panic!(
                        "Variable {} unexpectedly bound to {} in constraint {}.",
                        v.0,
                        x.to_polar(),
                        term.to_polar()
                    );
                }
            },
            (_, _) => panic!(
                "At least one side of a constraint expression must be a variable. This is {} {}",
                left.to_polar(),
                right.to_polar()
            ),
        }

        Ok(())
    }

    // TODO: non pub.
    pub fn constrain(&mut self, o: &Operation) -> PolarResult<()> {
        assert_eq!(o.operator, Operator::And, "bad constraint {}", o.to_polar());
        for var in o.variables() {
            match self.variable_state(&var) {
                VariableState::Bound(_) => (),
                _ => self.add_binding(&var, o.clone().into_term()),
            }
        }
        Ok(())
    }

    pub fn backtrack(&mut self, to: Bsp) {
        self.bindings.truncate(to)
    }

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

    // TODO maybe relevant_bindings
    // variable_bindings
    // bindings
    // bind_constants
    //
}

/// Get variable out of a term for ``add_constraint`` to determine where the
/// constraint is stored.
///
/// If the term is a variable, the variable is returned.
/// If the term is a dot expression, the VAR from VAR.field is returned.
/// Otherwise, the term is returned.
fn extract_variable(value: &Value) -> &Value {
    match value {
        Value::Variable(_) => value,
        Value::Expression(expr) if expr.operator == Operator::Dot => {
            extract_variable(expr.args[0].value())
        }
        _ => value,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn variable_state() {
        let mut bindings = BindingManager::new();

        let x = sym!("x");
        let y = sym!("y");
        let z = sym!("z");

        // Unbound.
        assert_eq!(bindings.variable_state(&x), VariableState::Unbound);

        // Bound.
        bindings.add_binding(&x, term!(1));
        assert_eq!(bindings.variable_state(&x), VariableState::Bound(term!(1)));

        bindings.add_binding(&x, term!(x.clone()));
        assert_eq!(bindings.variable_state(&x), VariableState::Cycle(vec![x.clone()]));

        // 2-cycle.
        bindings.add_binding(&x, term!(y.clone()));
        bindings.add_binding(&y, term!(x.clone()));
        assert_eq!(
            bindings.variable_state(&x),
            VariableState::Cycle(vec![x.clone(), y.clone()])
        );
        assert_eq!(
            bindings.variable_state(&y),
            VariableState::Cycle(vec![y.clone(), x.clone()])
        );

        // 3-cycle.
        bindings.add_binding(&x, term!(y.clone()));
        bindings.add_binding(&y, term!(z.clone()));
        bindings.add_binding(&z, term!(x.clone()));
        assert_eq!(
            bindings.variable_state(&x),
            VariableState::Cycle(vec![x.clone(), y.clone(), z.clone()])
        );
        assert_eq!(
            bindings.variable_state(&y),
            VariableState::Cycle(vec![y.clone(), z.clone(), x.clone()])
        );
        assert_eq!(
            bindings.variable_state(&z),
            VariableState::Cycle(vec![z.clone(), x.clone(), y])
        );

        // Expression.
        bindings.add_binding(&x, term!(op!(And)));
        assert_eq!(bindings.variable_state(&x), VariableState::Partial(op!(And)));
    }

}
