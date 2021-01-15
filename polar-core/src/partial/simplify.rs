use std::collections::HashSet;

use crate::folder::{fold_operation, fold_term, Folder};
use crate::kb::Bindings;
use crate::terms::{Operation, Operator, Symbol, Term, Value};

use super::partial::{invert_operation, FALSE, TRUE};

struct VariableSubber {
    this_var: Symbol,
}

impl VariableSubber {
    pub fn new(this_var: Symbol) -> Self {
        Self { this_var }
    }
}

impl Folder for VariableSubber {
    fn fold_variable(&mut self, v: Symbol) -> Symbol {
        if v == self.this_var {
            sym!("_this")
        } else {
            v
        }
    }

    fn fold_rest_variable(&mut self, v: Symbol) -> Symbol {
        if v == self.this_var {
            sym!("_this")
        } else {
            v
        }
    }
}

/// Substitute `sym!("_this")` for a variable in a partial.
pub fn sub_this(this: Symbol, term: Term) -> Term {
    if term
        .value()
        .as_symbol()
        .map(|s| s == &this)
        .unwrap_or(false)
    {
        return term;
    }
    fold_term(term, &mut VariableSubber::new(this))
}

/// Turn `_this = x` into `x` when it's ground.
fn simplify_trivial_constraint(this: Symbol, term: Term) -> Term {
    match term.value() {
        Value::Expression(o) if o.operator == Operator::Unify => {
            let left = &o.args[0];
            let right = &o.args[1];
            match (left.value(), right.value()) {
                (Value::Variable(v), Value::Variable(w))
                | (Value::Variable(v), Value::RestVariable(w))
                | (Value::RestVariable(v), Value::Variable(w))
                | (Value::RestVariable(v), Value::RestVariable(w))
                    if v == &this && w == &this =>
                {
                    TRUE.into_term()
                }
                (Value::Variable(l), _) | (Value::RestVariable(l), _)
                    if l == &this && right.is_ground() =>
                {
                    right.clone()
                }
                (_, Value::Variable(r)) | (_, Value::RestVariable(r))
                    if r == &this && left.is_ground() =>
                {
                    left.clone()
                }
                _ => term,
            }
        }
        _ => term,
    }
}

pub fn simplify_partial(var: &Symbol, term: Term) -> Term {
    let mut simplifier = Simplifier::new(var.clone());
    let simplified = simplifier.simplify_partial(term);
    let simplified = simplify_trivial_constraint(var.clone(), simplified);
    if matches!(simplified.value(), Value::Expression(e) if e.operator != Operator::And) {
        op!(And, simplified).into_term()
    } else {
        simplified
    }
}

/// Simplify the values of the bindings to be returned to the host language.
///
/// - For partials, simplify the constraint expressions.
/// - For non-partials, deep deref. TODO(ap/gj): deep deref.
pub fn simplify_bindings(bindings: Bindings) -> Option<Bindings> {
    let mut unsatisfiable = false;
    let mut simplify = |var: Symbol, term: Term| {
        let simplified = simplify_partial(&var, term);
        match simplified.value().as_expression() {
            Ok(o) if o == &FALSE => unsatisfiable = true,
            _ => (),
        }
        simplified
    };

    let bindings: Bindings = bindings
        .iter()
        .map(|(var, value)| match value.value() {
            Value::Expression(o) => {
                assert_eq!(o.operator, Operator::And);
                (var.clone(), simplify(var.clone(), value.clone()))
            }
            Value::Variable(v) | Value::RestVariable(v)
                if v.is_temporary_var()
                    && bindings.contains_key(v)
                    && matches!(bindings[v].value(), Value::Variable(_) | Value::RestVariable(_)) =>
            {
                (var.clone(), bindings[v].clone())
            }
            _ => (var.clone(), value.clone()),
        })
        .collect();

    if unsatisfiable {
        None
    } else {
        Some(bindings)
    }
}

pub struct Simplifier {
    bindings: Bindings,
    this_var: Symbol,
}

impl Folder for Simplifier {
    fn fold_term(&mut self, t: Term) -> Term {
        fold_term(self.deref(&t), self)
    }

    fn fold_operation(&mut self, mut o: Operation) -> Operation {
        if o.operator == Operator::And {
            // Preprocess constraints.
            let mut seen: HashSet<&Operation> = HashSet::new();
            o = o.clone_with_constraints(
                o.constraints()
                    .iter()
                    .filter(|o| *o != &TRUE) // Drop empty constraints.
                    .filter(|o| !seen.contains(&o.mirror()) && seen.insert(o)) // Deduplicate constraints.
                    .cloned()
                    .collect(),
            );
        }

        if o.operator == Operator::And || o.operator == Operator::Or {
            // Toss trivial unifications.
            o.args = o
                .constraints()
                .into_iter()
                .filter(|c| match c.operator {
                    Operator::Unify | Operator::Eq | Operator::Neq => {
                        assert_eq!(c.args.len(), 2);
                        let left = &c.args[0];
                        let right = &c.args[1];
                        left != right
                    }
                    _ => true,
                })
                .map(|c| c.into_term())
                .collect();
        }

        match o.operator {
            // Zero-argument conjunctions & disjunctions represent constants
            // TRUE and FALSE, respectively. We do not simplify them.
            Operator::And | Operator::Or if o.args.is_empty() => o,

            // Replace one-argument conjunctions & disjunctions with their argument.
            Operator::And | Operator::Or if o.args.len() == 1 => fold_operation(
                o.args[0]
                    .value()
                    .as_expression()
                    .expect("expression")
                    .clone(),
                self,
            ),

            // Non-trivial conjunctions. Choose a unification constraint to
            // make a binding from, maybe throw it away, and fold the rest.
            Operator::And if o.args.len() > 1 => {
                if let Some(i) = o.constraints().iter().position(|constraint| {
                    let other_constraints = o.clone_with_constraints(
                        o.constraints()
                            .into_iter()
                            .filter(|r| r != constraint)
                            .collect(),
                    );
                    let variables = other_constraints.variables();
                    self.maybe_bind_constraint(constraint, variables)
                }) {
                    o.args.remove(i);
                }
                fold_operation(o, self)
            }

            // Negation. Simplify the negated term, saving & restoring the
            // current bindings because bindings may not leak out of a negation.
            Operator::Not => {
                assert_eq!(o.args.len(), 1);
                let bindings = self.bindings.clone();
                let simplified = self.simplify_partial(o.args[0].clone());
                self.bindings = bindings;
                invert_operation(
                    simplified
                        .value()
                        .as_expression()
                        .expect("a simplified expression")
                        .clone(),
                )
            }

            // Default case.
            _ => fold_operation(o, self),
        }
    }
}

impl Simplifier {
    pub fn new(this_var: Symbol) -> Self {
        Self {
            this_var,
            bindings: Bindings::new(),
        }
    }

    pub fn bind(&mut self, var: Symbol, value: Term) {
        let new_value = self.deref(&value);
        if self.is_bound(&var) {
            let current_value = self.deref(&term!(var.clone()));
            if current_value.is_ground() && new_value.is_ground() {
                assert_eq!(&current_value, &new_value);
            }
        }

        self.bindings.insert(var, new_value);
    }

    pub fn deref(&self, term: &Term) -> Term {
        match term.value() {
            Value::Variable(var) | Value::RestVariable(var) => {
                self.bindings.get(var).unwrap_or(term).clone()
            }
            _ => term.clone(),
        }
    }

    fn is_bound(&self, var: &Symbol) -> bool {
        self.bindings.contains_key(var)
    }

    /// Term is a variable and the name = self.this_var
    fn is_this(&self, t: &Term) -> bool {
        match t.value() {
            Value::Variable(v) | Value::RestVariable(v) => v == &self.this_var,
            _ => false,
        }
    }

    /// Either _this or _this.?
    fn is_dot_this(&self, t: &Term) -> bool {
        match t.value() {
            Value::Expression(e) => e.operator == Operator::Dot && self.is_dot_this(&e.args[0]),
            _ => self.is_this(t),
        }
    }

    /// Returns true when the constraint can be replaced with a binding, and makes the binding.
    ///
    /// Params:
    ///     constraint: The constraint to consider removing from its parent.
    ///     other_variables: Variables referenced in the parent constraint by terms other than `constraint`.
    fn maybe_bind_constraint(
        &mut self,
        constraint: &Operation,
        other_variables: Vec<Symbol>,
    ) -> bool {
        match constraint.operator {
            // A conjunction of TRUE with X is X, so drop TRUE.
            Operator::And if constraint.args.is_empty() => true,

            // Choose a unification to maybe drop.
            Operator::Unify | Operator::Eq => {
                let left = &constraint.args[0];
                let right = &constraint.args[1];

                // Drop if the sides are exactly equal.
                left == right
                    // Or...
                    || match (left.value(), right.value()) {
                        // Bind l to _this or _this.? if:
                        // Variable(l) = _this.? AND l is referenced in another term
                        // Variable(l) = _this
                        (Value::Variable(l), _) | (Value::RestVariable(l), _)
                            if self.is_dot_this(right)
                                && (self.is_this(right) || other_variables.contains(l)) =>
                        {
                            self.bind(l.clone(), right.clone());
                            true
                        }
                        // _this = Variable(r)
                        (_, Value::Variable(r)) | (_, Value::RestVariable(r))
                            if self.is_dot_this(left)
                                && (self.is_this(left) || other_variables.contains(r)) =>
                        {
                            self.bind(r.clone(), left.clone());
                            true
                        }
                        // If either side is _this or _this.? don't drop the constraint.
                        _ if self.is_dot_this(left) || self.is_dot_this(right) => false,

                        // Both sides are variables, but neither is _this. Bind together.
                        (Value::Variable(l), Value::Variable(r))
                        | (Value::Variable(l), Value::RestVariable(r))
                        | (Value::RestVariable(l), Value::Variable(r))
                        | (Value::RestVariable(l), Value::RestVariable(r)) => {
                            self.bind(l.clone(), right.clone());
                            self.bind(r.clone(), left.clone());
                            true
                        }
                        // One side is a variable, the other is a ground value. Bind it.
                        (Value::Variable(l), _) | (Value::RestVariable(l), _) => {
                            self.bind(l.clone(), right.clone());
                            true
                        }
                        (_, Value::Variable(r)) | (_, Value::RestVariable(r)) => {
                            self.bind(r.clone(), left.clone());
                            true
                        }
                        _ => false,
                    }
            }
            _ => false,
        }
    }

    /// Simplify a partial until quiescence.
    pub fn simplify_partial(&mut self, mut term: Term) -> Term {
        let mut new;
        loop {
            new = self.fold_term(term.clone());
            if new == term {
                break;
            }
            term = new;
            self.bindings.clear();
        }
        new
    }
}
