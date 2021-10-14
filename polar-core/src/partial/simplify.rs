use std::collections::{HashMap, HashSet};

use crate::{bindings::Bindings, terms::*};

use super::partial::{invert_operation, FALSE, TRUE};

enum MaybeDrop {
    Keep,
    Drop,
    Bind(Symbol, Term),
    Check(Symbol, Term),
}

/// Substitute `sym!("_this")` for a variable in a partial.
pub fn sub_this(this: Symbol, term: Term) -> Term {
    use crate::folder::Folder;
    struct VariableSubber(HashMap<Symbol, Symbol>);

    impl Folder for VariableSubber {
        fn fold_variable(&mut self, v: Symbol) -> Symbol {
            if let Some(y) = self.0.get(&v) {
                y.clone()
            } else {
                v
            }
        }

        fn fold_rest_variable(&mut self, v: Symbol) -> Symbol {
            self.fold_variable(v)
        }
    }

    if term
        .value()
        .as_symbol()
        .map(|s| s == &this)
        .unwrap_or(false)
    {
        term
    } else {
        let mut map = HashMap::new();
        map.insert(this, sym!("_this"));
        VariableSubber(map).fold_term(term)
    }
}

/// Turn `_this = x` into `x` when it's ground.
fn simplify_trivial_constraint(term: Term) -> Term {
    match term.value() {
        Value::Expression(o) if o.operator == Operator::Unify => {
            let (left, right) = (&o.args[0], &o.args[1]);
            match (left.value(), right.value()) {
                (Value::Variable(v), Value::Variable(w))
                | (Value::Variable(v), Value::RestVariable(w))
                | (Value::RestVariable(v), Value::Variable(w))
                | (Value::RestVariable(v), Value::RestVariable(w))
                    if v == w =>
                {
                    TRUE.into()
                }
                (Value::Variable(_), _) | (Value::RestVariable(_), _) if right.is_ground() => {
                    right.clone()
                }
                (_, Value::Variable(_)) | (_, Value::RestVariable(_)) if left.is_ground() => {
                    left.clone()
                }
                _ => term,
            }
        }
        _ => term,
    }
}

pub fn simplify_partial(mut term: Term, output_vars: HashSet<Symbol>) -> Option<Term> {
    let mut simplifier = Simplifier::new(output_vars);
    simplifier.simplify_partial(&mut term)?;
    term = simplify_trivial_constraint(term);
    if matches!(term.value(), Value::Expression(e) if e.operator != Operator::And) {
        Some(op!(And, term).into())
    } else {
        Some(term)
    }
}

fn toss_trivial_unifies(args: &mut TermList) {
    args.retain(|c| {
        let o = c.value().as_expression().unwrap();
        match o.operator {
            Operator::Unify | Operator::Eq => {
                assert_eq!(o.args.len(), 2);
                let left = &o.args[0];
                let right = &o.args[1];
                left != right
            }
            _ => true,
        }
    });
}

fn simplify_var(all: bool, bindings: &Bindings, var: &Symbol, term: &Term) -> Option<Term> {
    let output_vars: HashSet<_> = bindings
        .keys()
        .filter(|v| all && *v == var || !all && !v.is_temporary_var())
        .cloned()
        .collect();

    match term.value() {
        Value::Variable(v) | Value::RestVariable(v)
            if !output_vars.contains(v)
                && bindings.contains_key(v)
                && matches!(
                    bindings[v].value(),
                    Value::Variable(_) | Value::RestVariable(_)
                ) =>
        {
            Some(bindings[v].clone())
        }
        Value::Expression(o) => {
            assert_eq!(o.operator, Operator::And);
            let simplified = simplify_partial(term.clone(), output_vars)?;
            match simplified.value().as_expression() {
                Ok(o) if o == &FALSE => None,
                _ => Some(simplified),
            }
        }
        _ => Some(term.clone()),
    }
}

/// Simplify the values of the bindings to be returned to the host language.
///
/// - For partials, simplify the constraint expressions.
/// - For non-partials, deep deref. TODO(ap/gj): deep deref.
pub fn simplify_bindings(bindings: Bindings, all: bool) -> Option<Bindings> {
    let b = bindings
        .iter()
        .filter_map(|(var, value)| {
            (all || !var.is_temporary_var())
                .then(|| simplify_var(all, &bindings, var, value).map(|s| (var.clone(), s)))
        })
        .collect();

    b
}

#[derive(Clone)]
struct Simplifier {
    bindings: Bindings,
    output_vars: HashSet<Symbol>,
    seen: HashSet<Term>,
}

type TermSimplifier = dyn Fn(&mut Simplifier, &mut Term) -> Option<()>; // cursed return type

impl Simplifier {
    fn new(output_vars: HashSet<Symbol>) -> Self {
        Self {
            output_vars,
            bindings: Bindings::new(),
            seen: HashSet::new(),
        }
    }

    fn bind(&mut self, var: Symbol, value: Term) -> Option<&mut Self> {
        match self.binding(&var) {
            None => {
                self.bindings.insert(var, self.deref(&value));
                Some(self)
            }
            Some(x) if *x == value => Some(self),
            _ => None,
        }
    }

    fn deref(&self, term: &Term) -> Term {
        match term.value() {
            Value::Variable(var) | Value::RestVariable(var) => {
                self.bindings.get(var).unwrap_or(term).clone()
            }
            _ => term.clone(),
        }
    }

    fn binding<'a>(&'a self, var: &Symbol) -> Option<&'a Term> {
        self.bindings.get(var)
    }

    fn is_bound(&self, var: &Symbol) -> bool {
        self.bindings.contains_key(var)
    }

    fn is_output(&self, t: &Term) -> bool {
        t.value()
            .as_symbol()
            .map_or(false, |v| self.output_vars.contains(v))
    }

    /// Determine whether to keep, drop, bind or conditionally bind a unification operation.
    ///
    /// Returns:
    /// - Keep: to indicate that the operation should not be removed
    /// - Drop: to indicate the operation should be removed with no new bindings
    /// - Bind(var, val) to indicate that the operation should be removed, and var should be
    ///                  bound to val.
    /// - Check(var, val) To indicate that the operation should be removed and var should
    ///                   be bound to val *if* var is referenced elsewhere in the expression.
    ///
    /// Params:
    ///     constraint: The constraint to consider removing from its parent.
    fn maybe_bind_constraint(&mut self, constraint: &Operation) -> MaybeDrop {
        use { Operator::*, Value::*, MaybeDrop::* };
        match constraint.operator {
            // X and X is always true, so drop.
            And if constraint.args.is_empty() => Drop,

            // Choose a unification to maybe drop.
            Unify | Eq => {
                let left = &constraint.args[0];
                let right = &constraint.args[1];

                if left == right {
                    // The sides are exactly equal, so drop.
                    Drop
                } else {
                    // Maybe bind one side to the other.
                    match (left.value(), right.value()) {
                        // Always keep unifications of two output variables (x = y).
                        (Variable(_), Variable(_))
                            if self.is_output(left) && self.is_output(right) =>
                        {
                            Keep
                        }
                        // Replace non-output variable l with right.
                        (Variable(l), _) if !self.is_bound(l) && !self.is_output(left) => {
                            Bind(l.clone(), right.clone())
                        }
                        // Replace non-output variable r with left.
                        (_, Variable(r)) if !self.is_bound(r) && !self.is_output(right) => {
                            Bind(r.clone(), left.clone())
                        }
                        // Replace unbound variable with ground value.
                        (Variable(var), val) if val.is_ground() && !self.is_bound(var) => {
                            Check(var.clone(), right.clone())
                        }
                        // Replace unbound variable with ground value.
                        (val, Variable(var)) if val.is_ground() && !self.is_bound(var) => {
                            Check(var.clone(), left.clone())
                        }
                        // Keep everything else.
                        _ => Keep,
                    }
                }
            }
            _ => Keep,
        }
    }

    /// Perform simplification of variable names in an operation by eliminating unification
    /// operations to express an operation in terms of output variables only.
    ///
    /// Also inverts negation operations.
    ///
    /// May require multiple calls to perform all eliminiations.
    fn simplify_operation_variables(
        &mut self,
        o: &mut Operation,
        simplify_term: &TermSimplifier,
    ) -> Option<()> {
        use { MaybeDrop::*, Operator::* };
        if o.operator == And || o.operator == Or {
            toss_trivial_unifies(&mut o.args);
        }

        match o.operator {
            // Zero-argument conjunctions & disjunctions represent constants
            // TRUE and FALSE, respectively. We do not simplify them.
            And | Or if o.args.is_empty() => (),

            // Replace one-argument conjunctions & disjunctions with their argument.
            And | Or if o.args.len() == 1 => {
                if let Value::Expression(operation) = o.args[0].value() {
                    *o = operation.clone();
                    self.simplify_operation_variables(o, simplify_term)?;
                }
            }

            // Non-trivial conjunctions. Choose unification constraints
            // to make bindings from and throw away; fold the rest.
            And if o.args.len() > 1 => {
                // Compute which constraints to keep.
                let mut keep: Vec<_> = o.args.iter().map(|_| true).collect();
                let mut references: Vec<_> = o.args.iter().map(|_| false).collect();
                for (i, arg) in o.args.iter().enumerate() {
                    match self.maybe_bind_constraint(arg.value().as_expression().unwrap()) {
                        Keep => (),
                        Drop => keep[i] = false,
                        Bind(var, value) => {
                            keep[i] = false;
                            self.bind(var, value)?;
                        }
                        Check(var, value) => {
                            for (j, arg) in o.args.iter().enumerate() {
                                if j != i && arg.contains_variable(&var) {
                                    self.bind(var, value)?;
                                    keep[i] = false;

                                    // record that this term references var and must be kept.
                                    references[j] = true;
                                    break;
                                }
                            }
                        }
                    }
                }

                // Drop the rest.
                let mut i = 0;
                o.args.retain(|_| {
                    i += 1;
                    keep[i - 1] || references[i - 1]
                });

                // Simplify the survivors.
                for arg in &mut o.args {
                    simplify_term(self, arg)?;
                }
            }

            // Negation. Simplify the negated term, saving & restoring the
            // current bindings because bindings may not leak out of a negation.
            Not => {
                assert_eq!(o.args.len(), 1);
                let mut simplified = o.args[0].clone();
                let mut simplifier = self.clone();
                simplifier.simplify_partial(&mut simplified);
                *o = invert_operation(
                    simplified
                        .value()
                        .as_expression()
                        .expect("a simplified expression")
                        .clone(),
                )
            }

            // Default case.
            _ => {
                for arg in &mut o.args {
                    simplify_term(self, arg)?;
                }
            }
        }

        // FIXME(gw) last minute consistency check hack. this shouldn't exist,
        // we should have failed already
        if matches!(o.operator, Eq | Unify) {
            let mut xs: Vec<_> = o.args.iter().filter(|x| x.is_ground()).collect();
            if let Some(x) = xs.pop() {
                for y in xs {
                    if x != y {
                        return None;
                    }
                }
            }
        }
        Some(())
    }

    /// Deduplicate an operation by removing terms that are mirrors or duplicates
    /// of other terms.
    fn deduplicate_operation(
        &mut self,
        o: &mut Operation,
        simplify_term: &TermSimplifier,
    ) -> Option<()> {
        fn preprocess_and(args: &mut TermList) {
            // HashSet of term hash values used to deduplicate. We use hash values
            // to avoid cloning to insert terms.
            let mut seen: HashSet<u64> = HashSet::with_capacity(args.len());
            args.retain(|a| {
                let o = a.value().as_expression().unwrap();
                o != &TRUE // trivial
                    && !seen.contains(&Term::from(o.mirror()).hash_value()) // reflection
                    && seen.insert(a.hash_value()) // duplicate
            });
        }

        if o.operator == Operator::And {
            preprocess_and(&mut o.args);
        }

        match o.operator {
            Operator::And | Operator::Or if o.args.is_empty() => (),

            // Replace one-argument conjunctions & disjunctions with their argument.
            Operator::And | Operator::Or if o.args.len() == 1 => {
                if let Value::Expression(operation) = o.args[0].value() {
                    *o = operation.clone();
                    self.deduplicate_operation(o, simplify_term)?;
                }
            }

            // Default case.
            _ => {
                for arg in &mut o.args {
                    simplify_term(self, arg)?;
                }
            }
        }

        Some(())
    }

    /// Simplify a term `term` in place by calling the simplification
    /// function `simplify_operation` on any Expression in that term.
    ///
    /// `simplify_operation` should perform simplification operations in-place
    /// on the operation argument. To recursively simplify sub-terms in that operation,
    /// it must call the passed TermSimplifier.
    fn simplify_term<F>(&mut self, term: &mut Term, simplify_operation: F) -> Option<()>
    where
        F: (Fn(&mut Self, &mut Operation, &TermSimplifier) -> Option<()>) + 'static + Clone,
    {
        if self.seen.contains(term) {
            return Some(());
        }

        let orig = term.clone();
        self.seen.insert(term.clone());
        *term = self.deref(term);

        match term.mut_value() {
            Value::Dictionary(dict) => {
                for (_, v) in dict.fields.iter_mut() {
                    self.simplify_term(v, simplify_operation.clone())?;
                }
            }
            Value::Call(call) => {
                for arg in call.args.iter_mut() {
                    self.simplify_term(arg, simplify_operation.clone())?;
                }
                if let Some(kwargs) = &mut call.kwargs {
                    for (_, v) in kwargs.iter_mut() {
                        self.simplify_term(v, simplify_operation.clone())?;
                    }
                }
            }
            Value::List(list) => {
                for elem in list.iter_mut() {
                    self.simplify_term(elem, simplify_operation.clone())?;
                }
            }
            Value::Expression(operation) => {
                let so = simplify_operation.clone();
                let cont = move |s: &mut Self, term: &mut Term| {
                    s.simplify_term(term, simplify_operation.clone())
                };
                so(self, operation, &cont)?;
            }
            _ => (),
        }

        if let Ok(sym) = orig.value().as_symbol() {
            if term.contains_variable(sym) {
                *term = orig.clone()
            }
        }
        self.seen.remove(&orig);
        Some(())
    }

    fn simplify_partial_loop(&mut self, term: &mut Term, hash: u64, len: usize) -> Option<()> {
        self.simplify_term(term, Simplifier::simplify_operation_variables)?;
        let (nhash, nlen) = (term.hash_value(), self.bindings.len());
        if hash == nhash && len == nlen {
            self.simplify_term(term, Simplifier::deduplicate_operation)
        } else {
            self.simplify_partial_loop(term, nhash, nlen)
        }
    }

    /// Simplify a partial until quiescence.
    fn simplify_partial(&mut self, term: &mut Term) -> Option<()> {
        // FIXME hash collisions
        self.simplify_partial_loop(term, term.hash_value(), self.bindings.len())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_simplify_circular_dot_with_isa() {
        let op = term!(op!(Dot, var!("x"), str!("x")));
        let op = term!(op!(Unify, var!("x"), op));
        let op = term!(op!(
            And,
            op,
            term!(op!(Isa, var!("x"), term!(pattern!(instance!("X")))))
        ));
        let mut vs: HashSet<Symbol> = HashSet::new();
        vs.insert(sym!("x"));
        let x = simplify_partial(op, vs).unwrap();
        assert_eq!(
            x,
            term!(op!(
                And,
                term!(op!(Unify, var!("x"), term!(op!(Dot, var!("x"), str!("x"))))),
                term!(op!(Isa, var!("x"), term!(pattern!(instance!("X")))))
            ))
        );
    }
}
