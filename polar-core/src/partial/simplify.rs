use std::collections::HashSet;

use crate::folder::{fold_operation, fold_partial, fold_term, Folder};
use crate::formatting::ToPolarString;
use crate::kb::Bindings;
// use crate::terms::{Operation, Operator, Symbol, Term, TermList, Value};
use crate::terms::{Operation, Operator, Symbol, Term, Value};

use super::Partial;

fn sub_this(partial: Partial, this_var: Symbol) -> Partial {
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
    }

    fold_partial(partial, &mut VariableSubber::new(this_var))
}

/// Simplify the values of the bindings to be returned to the host language.
///
/// - For partials, simplify the constraint expressions.
/// - For non-partials, deep deref.
pub fn simplify_bindings(bindings: Bindings) -> Bindings {
    let bs = bindings.clone();
    bindings
        .into_iter()
        .map(|(var, value)| match value.value() {
            Value::Partial(partial) => {
                let subbed = sub_this(partial.clone(), var.clone());
                let simplified = simplify_partial(subbed.into_term(), var.clone(), bs.clone());
                // assert!(simplified.value().as_expression().is_ok());

                (var, simplified)
            }
            _ => (var, value),
        })
        .collect()
}

pub struct Simplifier {
    bindings: Bindings,
    var: Symbol,
}

impl Folder for Simplifier {
    fn fold_term(&mut self, t: Term) -> Term {
        match t.value() {
            Value::Partial(p) if p.constraints().len() == 1 => {
                let c = &p.constraints()[0];
                if c.operator == Operator::Unify && c.args.len() == 2 {
                    let left = &c.args[0];
                    let right = &c.args[1];

                    match (left.value(), right.value()) {
                        // (Value::Variable(left_var), Value::Variable(right_var))
                        //     if left_var.is_this_var() && right_var.is_this_var() =>
                        // {
                        //     unreachable!()
                        // }
                        (Value::Variable(_), Value::Variable(_)) => fold_term(t, self),
                        (Value::Variable(v), _) if v.is_this_var() => {
                            fold_term(right.clone(), self)
                        }
                        (_, Value::Variable(v)) if v.is_this_var() => fold_term(left.clone(), self),
                        _ => fold_term(t, self),
                    }
                } else {
                    fold_term(t, self)
                }
            }
            _ => fold_term(t, self),
        }
    }

    fn fold_partial(&mut self, p: Partial) -> Partial {
        let mut seen: HashSet<&Operation> = HashSet::new();
        let compacted_constraints = p
            .constraints
            .iter()
            .filter(|&c| c != &op!(And)) // Drop empty constraints.
            .filter(|o| seen.insert(o)) // Deduplicate constraints.
            .cloned()
            .collect();
        fold_partial(p.clone_with_constraints(compacted_constraints), self)
    }

    fn fold_operation(&mut self, o: Operation) -> Operation {
        match o.operator {
            Operator::Unify => {
                let left = &o.args[0];
                let right = &o.args[1];

                match (left.value(), right.value()) {
                    (Value::Variable(Symbol(left)), Value::Variable(Symbol(right)))
                        if left == "_this" && right == "_this" =>
                    {
                        op!(And)
                    }
                    (Value::Variable(Symbol(v)), Value::Variable(other)) if v == "_this" => {
                        if other == &self.var {
                            eprintln!("A.1 {} = {}", left.to_polar(), right.to_polar());
                        } else {
                            // let other_term = &self.bindings[other];
                            // if other_term == &self.bindings[&self.var] {
                            eprintln!("A.2 {} = {}", left.to_polar(), right.to_polar());
                            //     op!(And)
                            // } else {
                            //     eprintln!("A.3 {} = {}", left.to_polar(), right.to_polar());
                            //     op!(Unify, left.clone(), fold_term(other_term.clone(), self))
                            // }
                            self.bindings.insert(other.clone(), term!(sym!("_this")));
                        }
                        op!(And)
                    }
                    (Value::Variable(other), Value::Variable(Symbol(v))) if v == "_this" => {
                        if other == &self.var {
                            eprintln!("B.1 {} = {}", left.to_polar(), right.to_polar());
                        } else {
                            // let other_term = &self.bindings[other];
                            // if other_term == &self.bindings[&self.var] {
                            eprintln!("B.2 {} = {}", left.to_polar(), right.to_polar());
                            //     op!(And)
                            // } else {
                            //     eprintln!("B.3 {} = {}", left.to_polar(), right.to_polar());
                            //     op!(Unify, fold_term(other_term.clone(), self), right.clone())
                            // }
                            self.bindings.insert(other.clone(), term!(sym!("_this")));
                        }
                        op!(And)
                    }
                    (Value::Variable(Symbol(v)), _) if v == "_this" => {
                        eprintln!("C.1 {} = {}", left.to_polar(), right.to_polar());
                        fold_operation(op!(Unify, left.clone(), right.clone()), self)
                    }
                    (_, Value::Variable(Symbol(v))) if v == "_this" => {
                        eprintln!("D.1 {} = {}", left.to_polar(), right.to_polar());
                        fold_operation(op!(Unify, left.clone(), right.clone()), self)
                    }
                    (Value::Variable(_), Value::Variable(_)) => {
                        let left = self.deref(left);
                        let right = self.deref(right);
                        if left == right {
                            eprintln!("E.1 {} = {}", left.to_polar(), right.to_polar());
                            op!(And)
                        } else {
                            eprintln!("E.2 {} = {}", left.to_polar(), right.to_polar());
                            fold_operation(op!(Unify, left, right), self)
                        }
                    }
                    (Value::Partial(left_partial), Value::Partial(right_partial)) => {
                        if left_partial == right_partial {
                            eprintln!("F.1 {} = {}", left.to_polar(), right.to_polar());
                            op!(And)
                        } else {
                            panic!("F.2 {} = {}", left.to_polar(), right.to_polar());
                        }
                    }
                    (Value::Variable(v), _) => {
                        eprintln!("G.1 {} = {}", left.to_polar(), right.to_polar());
                        let bound_term = &self.bindings[v];
                        if &self.bindings[&self.var] == bound_term {
                            self.bindings.insert(v.clone(), right.clone());
                            op!(And)
                        } else {
                            fold_operation(op!(Unify, bound_term.clone(), right.clone()), self)
                        }
                    }
                    // TODO(gj): unify_var ordering
                    (_, Value::Variable(_)) => {
                        panic!("H.1 {} = {}", left.to_polar(), right.to_polar())
                    }
                    _ => fold_operation(o, self),
                }
            }
            Operator::Dot => {
                let object = &o.args[0];
                let field = &o.args[1];
                match object.value() {
                    Value::Variable(v) => {
                        if let Some(Value::Variable(other)) =
                            self.bindings.get(v).map(|t| t.value())
                        {
                            if other.is_this_var() {
                                fold_operation(op!(Dot, term!(sym!("_this")), field.clone()), self)
                            } else {
                                fold_operation(o, self)
                            }
                        } else {
                            fold_operation(o, self)
                        }
                    }
                    _ => fold_operation(o, self),
                }
            }
            _ => fold_operation(o, self),
        }
    }

    // fn fold_operation(&mut self, mut o: Operation) -> Operation {
    //     if let Some(op) = self.maybe_unwrap_single_argument_and_or(&o) {
    //         o = op;
    //     }
    //
    //     match o.operator {
    //         Operator::Neq => {
    //             let left = &o.args[0];
    //             let right = &o.args[1];
    //             Operation {
    //                 operator: Operator::And,
    //                 args: match (left.value(), right.value()) {
    //                     // Distribute **inverted** expression over the partial.
    //                     (Value::Partial(c), Value::Expression(_)) => {
    //                         self.map_constraints(&c.inverted_constraints(0), right)
    //                     }
    //                     (Value::Expression(_), Value::Partial(c)) => {
    //                         self.map_constraints(&c.inverted_constraints(0), left)
    //                     }
    //                     _ => return fold_operation(o, self),
    //                 },
    //             }
    //         }
    //         Operator::Eq | Operator::Unify => {
    //             let left = &o.args[0];
    //             let right = &o.args[1];
    //             Operation {
    //                 operator: Operator::And,
    //                 args: match (left.value(), right.value()) {
    //                     // Distribute expression over the partial.
    //                     (Value::Partial(c), Value::Expression(_)) => {
    //                         self.map_constraints(c.constraints(), right)
    //                     }
    //                     (Value::Expression(_), Value::Partial(c)) => {
    //                         self.map_constraints(c.constraints(), left)
    //                     }
    //                     _ => return fold_operation(o, self),
    //                 },
    //             }
    //         }
    //         _ => fold_operation(o, self),
    //     }
    // }
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
    pub fn new(var: Symbol, bindings: Bindings) -> Self {
        Self { var, bindings }
    }

    pub fn deref(&self, term: &Term) -> Term {
        match term.value() {
            // TODO(gj): RestVariable?
            Value::Variable(symbol) => {
                if let Some(value) = self.bindings.get(symbol) {
                    if value != term {
                        return self.deref(value);
                    }
                }
                term.clone()
            }
            _ => term.clone(),
        }
    }

    // /// If `operation` is a 1-arg AND or OR operation, return its argument.
    // ///
    // /// Returns: Some(op) if a rewrite occurred; otherwise None.
    // fn maybe_unwrap_single_argument_and_or(&self, operation: &Operation) -> Option<Operation> {
    //     match operation {
    //         // Unwrap a single-arg And or Or expression and fold the inner term.
    //         Operation {
    //             operator: Operator::And,
    //             args,
    //         }
    //         | Operation {
    //             operator: Operator::Or,
    //             args,
    //         } if args.len() == 1 => {
    //             if let Value::Expression(op) = args[0].value() {
    //                 Some(op.clone())
    //             } else {
    //                 None
    //             }
    //         }
    //         _ => None,
    //     }
    // }

    // /// Substitute the this variable in a constraint with a dot operation.
    // /// Given `this` and `x`, return `x`.
    // /// Given `this.x` and `this.y`, return `this.x.y`.
    // fn sub_this(arg: &Term, dot_op: &Term) -> Term {
    //     assert!(matches!(
    //         dot_op.value(),
    //         Value::Expression(Operation {
    //             operator: Operator::Dot,
    //             ..
    //         })
    //     ));
    //
    //     match arg.value() {
    //         Value::Variable(v) if v.is_this_var() => dot_op.clone(),
    //         Value::Expression(Operation { operator, args }) => {
    //             arg.clone_with_value(Value::Expression(Operation {
    //                 operator: *operator,
    //                 args: args.iter().map(|arg| Self::sub_this(arg, dot_op)).collect(),
    //             }))
    //         }
    //         _ => arg.clone(),
    //     }
    // }

    // /// Substitute the this variable in a list of constraints with a dot operation path.
    // fn map_constraints(&mut self, constraints: &[Operation], dot_op: &Term) -> TermList {
    //     constraints
    //         .iter()
    //         .map(|c| Operation {
    //             operator: c.operator,
    //             args: c
    //                 .args
    //                 .iter()
    //                 .map(|arg| Self::sub_this(arg, dot_op))
    //                 .collect(),
    //         })
    //         .map(|c| dot_op.clone_with_value(Value::Expression(fold_operation(c, self))))
    //         .collect()
    // }
}

/// Simplify a partial until quiescence.
fn simplify_partial(mut term: Term, var: Symbol, bindings: Bindings) -> Term {
    let mut simplifier = Simplifier::new(var.clone(), bindings);
    let mut new;
    loop {
        eprintln!("SIMPLIFYING: {} => {}", var, term.to_polar());
        new = simplifier.fold_term(term.clone());
        if new == term {
            break;
        }
        term = new;
    }

    // let new = fold_term(new, &mut EmptyAndRemover {});

    let mut partial_to_expr = PartialToExpression {};

    partial_to_expr.fold_term(new)
}

// #[cfg(test)]
// mod test {
//     use super::*;
//     use crate::terms::*;
//
//     macro_rules! assert_expr_eq {
//         ($left:expr, $right:expr) => {{
//             let left = $left;
//             let right = $right;
//
//             assert_eq!(
//                 left.clone(),
//                 right.clone(),
//                 "{} != {}",
//                 left.to_polar(),
//                 right.to_polar()
//             );
//         }};
//     }
//
//     #[test]
//     fn test_simplify_non_partial() {
//         let nonpartial = term!(btreemap! {
//             sym!("a") => term!(1),
//             sym!("b") => term!([
//                 value!("hello")
//             ]),
//         });
//         assert_eq!(simplify_partial(nonpartial.clone()), nonpartial);
//     }
//
//     #[test]
//     // TODO(gj): Is this maybe a silly test now that we don't simplify "trivial" unifications?
//     fn test_simplify_partial() {
//         let partial = term!(Partial {
//             variable: sym!("a"),
//             constraints: vec![op!(And, term!(op!(Unify, term!(sym!("_this")), term!(1))))],
//         });
//         assert_eq!(
//             simplify_partial(partial),
//             term!(op!(Unify, term!(sym!("_this")), term!(1)))
//         );
//     }
//
//     #[test]
//     fn test_simplify_single_item_and() {
//         let partial = term!(partial!(
//             "a",
//             [op!(And, term!(op!(Eq, term!(1), term!(2))))]
//         ));
//         assert_eq!(
//             simplify_partial(partial),
//             term!(op!(Eq, term!(1), term!(2)))
//         );
//
//         let partial = term!(partial!(
//             "a",
//             [op!(
//                 And,
//                 term!(op!(And, term!(op!(Eq, term!(1), term!(2)))))
//             )]
//         ));
//         assert_eq!(
//             simplify_partial(partial),
//             term!(op!(Eq, term!(1), term!(2)))
//         );
//
//         let partial = term!(partial!(
//             "a",
//             [op!(
//                 And,
//                 term!(op!(Eq, term!(3), term!(4))),
//                 term!(op!(And, term!(op!(Eq, term!(1), term!(2)))))
//             )]
//         ));
//
//         assert_expr_eq!(
//             simplify_partial(partial),
//             term!(op!(
//                 And,
//                 term!(op!(Eq, term!(3), term!(4))),
//                 term!(op!(Eq, term!(1), term!(2)))
//             ))
//         );
//     }
// }
