use std::collections::HashSet;

use crate::folder::{fold_operation, fold_partial, fold_term, Folder};
use crate::kb::Bindings;
use crate::terms::{Operation, Operator, Term, TermList, Value};

use super::Partial;

/// Simplify the values of the bindings to be returned to the host language.
///
/// - For partials, simplify the constraint expressions.
/// - For non-partials, deep deref.
pub fn simplify_bindings(bindings: Bindings) -> Bindings {
    bindings
        .into_iter()
        .map(|(var, value)| match value.value() {
            Value::Partial(_) => {
                let simplified = simplify_partial(value);
                assert!(simplified.value().as_expression().is_ok());

                (var, simplified)
            }
            _ => (var, value),
        })
        .collect()
}

pub struct Simplifier;
impl Folder for Simplifier {
    /// Deduplicate constraints.
    fn fold_partial(&mut self, partial: Partial) -> Partial {
        fold_partial(self.deduplicate_constraints(partial), self)
    }

    fn fold_operation(&mut self, mut o: Operation) -> Operation {
        if let Some(op) = self.maybe_unwrap_single_argument_and_or(&o) {
            o = op;
        }

        match o.operator {
            Operator::Neq => {
                let left = &o.args[0];
                let right = &o.args[1];
                Operation {
                    operator: Operator::And,
                    args: match (left.value(), right.value()) {
                        // Distribute **inverted** expression over the partial.
                        (Value::Partial(c), Value::Expression(_)) => {
                            self.map_constraints(&c.inverted_constraints(0), right)
                        }
                        (Value::Expression(_), Value::Partial(c)) => {
                            self.map_constraints(&c.inverted_constraints(0), left)
                        }
                        _ => return fold_operation(o, self),
                    },
                }
            }
            Operator::Eq | Operator::Unify => {
                let left = &o.args[0];
                let right = &o.args[1];
                Operation {
                    operator: Operator::And,
                    args: match (left.value(), right.value()) {
                        // Distribute expression over the partial.
                        (Value::Partial(c), Value::Expression(_)) => {
                            self.map_constraints(c.constraints(), right)
                        }
                        (Value::Expression(_), Value::Partial(c)) => {
                            self.map_constraints(c.constraints(), left)
                        }
                        _ => return fold_operation(o, self),
                    },
                }
            }
            _ => fold_operation(o, self),
        }
    }
}

struct PartialToExpression;
impl Folder for PartialToExpression {
    fn fold_term(&mut self, t: Term) -> Term {
        match t.value() {
            Value::Partial(partial) => fold_term(partial.clone().into_expression(), self),
            _ => fold_term(t, self),
        }
    }
}

impl Simplifier {
    /// Remove duplicate constraints from a partial.
    fn deduplicate_constraints(&mut self, partial: Partial) -> Partial {
        let mut seen: HashSet<&Operation> = HashSet::new();
        let constraints = partial
            .constraints()
            .iter()
            .filter(|o| seen.insert(o))
            .cloned()
            .collect();
        partial.clone_with_constraints(constraints)
    }

    /// If ``operation`` is an AND or OR operation with 1 argument, return the 1st argument.
    ///
    /// Returns: Some(op) if a rewrite occured, or None.
    fn maybe_unwrap_single_argument_and_or(&self, operation: &Operation) -> Option<Operation> {
        match operation {
            // Unwrap a single-arg And or Or expression and fold the inner term.
            Operation {
                operator: Operator::And,
                args,
            }
            | Operation {
                operator: Operator::Or,
                args,
            } if args.len() == 1 => {
                if let Value::Expression(op) = args[0].value() {
                    Some(op.clone())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Subsitute the this variable in a constraint with a dot operation.
    /// Given `this` and `x`, return `x`.
    /// Given `this.x` and `this.y`, return `this.x.y`.
    fn sub_this(arg: &Term, dot_op: &Term) -> Term {
        assert!(matches!(
            dot_op.value(),
            Value::Expression(Operation {
                operator: Operator::Dot,
                ..
            })
        ));

        match arg.value() {
            Value::Variable(v) if v.is_this_var() => dot_op.clone(),
            Value::Expression(Operation { operator, args }) => {
                arg.clone_with_value(Value::Expression(Operation {
                    operator: *operator,
                    args: args.iter().map(|arg| Self::sub_this(arg, dot_op)).collect(),
                }))
            }
            _ => arg.clone(),
        }
    }

    /// Substitute the _this variable in a list of constraints with a dot operation path.
    fn map_constraints(&mut self, constraints: &[Operation], dot_op: &Term) -> TermList {
        constraints
            .iter()
            .map(|c| Operation {
                operator: c.operator,
                args: c
                    .args
                    .iter()
                    .map(|arg| Self::sub_this(arg, dot_op))
                    .collect(),
            })
            .map(|c| dot_op.clone_with_value(Value::Expression(fold_operation(c, self))))
            .collect()
    }
}

/// Simplify a partial until quiescence.
fn simplify_partial(mut term: Term) -> Term {
    let mut simplifier = Simplifier {};
    let mut new;
    loop {
        new = simplifier.fold_term(term.clone());
        if new == term {
            break;
        }
        term = new;
    }

    let mut partial_to_expr = PartialToExpression {};

    partial_to_expr.fold_term(new)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::terms::*;

    macro_rules! assert_expr_eq {
        ($left:expr, $right:expr) => {{
            let left = $left;
            let right = $right;

            assert_eq!(
                left.clone(),
                right.clone(),
                "{} != {}",
                left.to_polar(),
                right.to_polar()
            );
        }};
    }

    #[test]
    fn test_simplify_non_partial() {
        let nonpartial = term!(btreemap! {
            sym!("a") => term!(1),
            sym!("b") => term!([
                value!("hello")
            ]),
        });
        assert_eq!(simplify_partial(nonpartial.clone()), nonpartial);
    }

    #[test]
    // TODO(gj): Is this maybe a silly test now that we don't simplify "trivial" unifications?
    fn test_simplify_partial() {
        let partial = term!(Partial {
            variable: sym!("a"),
            constraints: vec![op!(And, term!(op!(Unify, term!(sym!("_this")), term!(1))))],
        });
        assert_eq!(
            simplify_partial(partial),
            term!(op!(Unify, term!(sym!("_this")), term!(1)))
        );
    }

    #[test]
    fn test_simplify_single_item_and() {
        let partial = term!(partial!(
            "a",
            [op!(And, term!(op!(Eq, term!(1), term!(2))))]
        ));
        assert_eq!(
            simplify_partial(partial),
            term!(op!(Eq, term!(1), term!(2)))
        );

        let partial = term!(partial!(
            "a",
            [op!(
                And,
                term!(op!(And, term!(op!(Eq, term!(1), term!(2)))))
            )]
        ));
        assert_eq!(
            simplify_partial(partial),
            term!(op!(Eq, term!(1), term!(2)))
        );

        let partial = term!(partial!(
            "a",
            [op!(
                And,
                term!(op!(Eq, term!(3), term!(4))),
                term!(op!(And, term!(op!(Eq, term!(1), term!(2)))))
            )]
        ));

        assert_expr_eq!(
            simplify_partial(partial),
            term!(op!(
                And,
                term!(op!(Eq, term!(3), term!(4))),
                term!(op!(Eq, term!(1), term!(2)))
            ))
        );
    }
}
