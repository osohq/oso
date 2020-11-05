use std::collections::HashSet;

use crate::folder::{fold_constraints, fold_operation, fold_term, Folder};
use crate::kb::Bindings;
use crate::terms::{Operation, Operator, Term, TermList, Value};

use super::Constraints;

/// Simplify the values of the bindings to be returned to the host language.
///
/// - For partials, simplify the constraint expressions.
/// - For non-partials, deep deref.
pub fn simplify_bindings(bindings: Bindings) -> Bindings {
    bindings
        .into_iter()
        .map(|(var, value)| match value.value() {
            Value::Partial(_) => {
                let mut simplified = simplify_partial(value);
                if let Value::Partial(partial) = simplified.value() {
                    simplified = partial.clone().into_expression();
                }
                (var, simplified)
            }
            _ => (var, value),
        })
        .collect()
}

pub struct Simplifier;
impl Folder for Simplifier {
    fn fold_term(&mut self, t: Term) -> Term {
        fn maybe_unwrap_operation(o: &Operation) -> Option<Term> {
            match o {
                // Unwrap a single-arg And or Or expression and fold the inner term.
                Operation {
                    operator: Operator::And,
                    args,
                }
                | Operation {
                    operator: Operator::Or,
                    args,
                } if args.len() == 1 => Some(args[0].clone()),
                _ => None,
            }
        }

        match t.value() {
            Value::Expression(o) => fold_term(maybe_unwrap_operation(o).unwrap_or(t), self),

            // Elide partial when its constraints are trivial.
            Value::Partial(Constraints { operations, .. }) if operations.len() == 1 => {
                fold_term(maybe_unwrap_operation(&operations[0]).unwrap_or(t), self)
            }

            _ => fold_term(t, self),
        }
    }

    /// Deduplicate constraints.
    fn fold_constraints(&mut self, c: Constraints) -> Constraints {
        let mut seen: HashSet<&Operation> = HashSet::new();
        let ops = c
            .operations
            .iter()
            .filter(|o| seen.insert(o))
            .cloned()
            .collect();
        fold_constraints(c.clone_with_operations(ops), self)
    }

    fn fold_operation(&mut self, o: Operation) -> Operation {
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

        // Optionally sub `expr` into each of the arguments of the partial's operations.
        let mut map_ops = |partial_ops: &[Operation], expr: &Term| -> TermList {
            partial_ops
                .iter()
                .map(|o| Operation {
                    operator: o.operator,
                    args: o.args.iter().map(|arg| sub_this(arg, expr)).collect(),
                })
                .map(|o| expr.clone_with_value(Value::Expression(fold_operation(o, self))))
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
                            map_ops(&c.inverted_operations(0), right)
                        }
                        (Value::Expression(_), Value::Partial(c)) => {
                            map_ops(&c.inverted_operations(0), left)
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
                        (Value::Partial(c), Value::Expression(_)) => map_ops(c.operations(), right),
                        (Value::Expression(_), Value::Partial(c)) => map_ops(c.operations(), left),
                        _ => return fold_operation(o, self),
                    },
                }
            }
            _ => fold_operation(o, self),
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
    new
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::terms::*;

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
        let partial = term!(Constraints {
            variable: sym!("a"),
            operations: vec![op!(And, term!(op!(Unify, term!(sym!("_this")), term!(1))))],
        });
        assert_eq!(
            simplify_partial(partial),
            term!(op!(Unify, term!(sym!("_this")), term!(1)))
        );
    }
}
