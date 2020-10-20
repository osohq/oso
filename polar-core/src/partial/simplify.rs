use super::Constraints;

use crate::folder::{fold_operation, fold_term, Folder};
use crate::kb::Bindings;
use crate::terms::{Operation, Operator, Term, TermList, Value};

/// Simplify the values of the bindings to be returned to the host language.
///
/// - For partials, simplify the constraint expressions.
/// - For non-partials, deep deref.
pub fn simplify_bindings(bindings: Bindings) -> Bindings {
    bindings
        .into_iter()
        .filter(|(v, _)| !v.is_temporary_var())
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
        match t.value() {
            Value::Partial(Constraints { operations, .. }) if operations.len() == 1 => {
                fn is_this_arg(t: &Term) -> bool {
                    matches!(t.value(), Value::Variable(v) if v.is_this_var())
                }

                match operations.get(0).unwrap() {
                    Operation {
                        operator: Operator::And,
                        args,
                    } if args.len() == 1 => fold_term(args.get(0).unwrap().clone(), self),

                    Operation {
                        operator: Operator::Unify,
                        args,
                    } if args.iter().any(is_this_arg) => {
                        let mut args = args
                            .iter()
                            .filter(|arg| !is_this_arg(arg))
                            .cloned()
                            .collect::<TermList>();
                        assert_eq!(args.len(), 1);
                        args.pop().unwrap()
                    }
                    _ => fold_term(t, self),
                }
            }
            _ => fold_term(t, self),
        }
    }

    fn fold_operation(&mut self, o: Operation) -> Operation {
        /// Given `_this` and `x`, return `x`.
        /// Given `_this.x` and `_this.y`, return `_this.x.y`.
        fn sub_this(term: &Term, replacement: &Term) -> Term {
            match (term.value(), replacement.value()) {
                (Value::Variable(v), _) if v.is_this_var() => replacement.clone(),
                (
                    Value::Expression(Operation {
                        operator: Operator::Dot,
                        args,
                    }),
                    Value::Expression(Operation {
                        operator: Operator::Dot,
                        ..
                    }),
                ) => term.clone_with_value(Value::Expression(Operation {
                    operator: Operator::Dot,
                    args: vec![replacement.clone(), args.get(1).unwrap().clone()],
                })),
                _ => term.clone(),
            }
        }

        // Optionally sub `replacement` into each of the arguments of the operations.
        let mut map_ops = |ops: &[Operation], replacement: &Term| -> TermList {
            ops.iter()
                .map(|o| Operation {
                    operator: o.operator,
                    args: o.args.iter().map(|a| sub_this(a, replacement)).collect(),
                })
                .map(|o| replacement.clone_with_value(Value::Expression(fold_operation(o, self))))
                .collect()
        };

        match o.operator {
            Operator::Unify => {
                let left = o.args.get(0).unwrap();
                let right = o.args.get(1).unwrap();
                Operation {
                    operator: Operator::And,
                    args: match (left.value(), right.value()) {
                        // Distribute expression over the partial.
                        (Value::Partial(c), Value::Expression(_)) => map_ops(&c.operations, right),
                        (Value::Expression(_), Value::Partial(c)) => map_ops(&c.operations, left),
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
