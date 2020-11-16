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
        let mut seen: HashSet<&Operation> = HashSet::new();
        let ops = partial
            .constraints()
            .iter()
            .filter(|o| seen.insert(o))
            .cloned()
            .collect();
        fold_partial(partial.clone_with_constraints(ops), self)
    }

    fn fold_operation(&mut self, mut o: Operation) -> Operation {
        fn maybe_unwrap_operation(o: &Operation) -> Option<Operation> {
            match o {
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

        if let Some(op) = maybe_unwrap_operation(&o) {
            o = op;
        }

        /// Given `this` and `x`, return `x`.
        /// Given `this.x` and `this.y`, return `this.x.y`.
        fn sub_this(arg: &Term, expr: &Term) -> Term {
            match (arg.value(), expr.value()) {
                (Value::Variable(v), _) if v.is_this_var() => expr.clone(),
                (
                    Value::Expression(Operation {
                        operator: Operator::Dot,
                        args,
                    }),
                    Value::Expression(Operation {
                        operator: Operator::Dot,
                        ..
                    }),
                ) => arg.clone_with_value(Value::Expression(Operation {
                    operator: Operator::Dot,
                    args: vec![expr.clone(), args[1].clone()],
                })),
                _ => arg.clone(),
            }
        }

        // Optionally sub `expr` into each of the arguments of the partial's constraints.
        let mut map_constraints = |constraints: &[Operation], expr: &Term| -> TermList {
            constraints
                .iter()
                .map(|c| Operation {
                    operator: c.operator,
                    args: c.args.iter().map(|arg| sub_this(arg, expr)).collect(),
                })
                .map(|c| expr.clone_with_value(Value::Expression(fold_operation(c, self))))
                .collect()
        };

        match o.operator {
            Operator::Neq => {
                let left = &o.args[0];
                let right = &o.args[1];
                Operation {
                    operator: Operator::And,
                    args: match (left.value(), right.value()) {
                        // Distribute **inverted** expression over the partial.
                        (Value::Partial(c), Value::Expression(_)) => {
                            map_constraints(&c.inverted_constraints(0), right)
                        }
                        (Value::Expression(_), Value::Partial(c)) => {
                            map_constraints(&c.inverted_constraints(0), left)
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
                            map_constraints(c.constraints(), right)
                        }
                        (Value::Expression(_), Value::Partial(c)) => {
                            map_constraints(c.constraints(), left)
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
