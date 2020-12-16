use std::collections::{HashMap, HashSet};

use crate::folder::{fold_operation, fold_partial, fold_term, Folder};
use crate::formatting::ToPolarString;
use crate::kb::Bindings;
// use crate::terms::{Operation, Operator, Symbol, Term, TermList, Value};
use crate::terms::{Operation, Operator, Symbol, Term, Value};

use super::Partial;

/// A trivially true expression.
const TRUE: Operation = op!(And);

/// Invert operators.
fn invert_operation(Operation { operator, args }: Operation) -> Operation {
    fn invert_args(args: Vec<Term>) -> Vec<Term> {
        args.into_iter()
            .map(|t| {
                t.clone_with_value(value!(invert_operation(
                    t.value().as_expression().unwrap().clone()
                )))
            })
            .collect()
    }

    match operator {
        Operator::And => Operation {
            operator: Operator::Or,
            args: invert_args(args),
        },
        Operator::Or => Operation {
            operator: Operator::And,
            args: invert_args(args),
        },
        Operator::Unify | Operator::Eq => Operation {
            operator: Operator::Neq,
            args,
        },
        Operator::Neq => Operation {
            operator: Operator::Unify,
            args,
        },
        Operator::Gt => Operation {
            operator: Operator::Leq,
            args,
        },
        Operator::Geq => Operation {
            operator: Operator::Lt,
            args,
        },
        Operator::Lt => Operation {
            operator: Operator::Geq,
            args,
        },
        Operator::Leq => Operation {
            operator: Operator::Gt,
            args,
        },
        Operator::Debug | Operator::Print | Operator::New | Operator::Dot => {
            Operation { operator, args }
        }
        Operator::Isa => Operation {
            operator: Operator::Not,
            args: vec![term!(op!(Isa, args[0].clone(), args[1].clone()))],
        },
        Operator::Not => args[0]
            .value()
            .as_expression()
            .expect("negated expression")
            .clone(),
        _ => todo!("negate {:?}", operator),
    }
}

/// Simplify the values of the bindings to be returned to the host language.
///
/// - For partials, simplify the constraint expressions.
/// - For non-partials, deep deref.
/// TODO(ap): deep deref.
pub fn simplify_bindings(bindings: Bindings) -> Bindings {
    bindings
        .into_iter()
        .map(|(var, value)| match value.value() {
            Value::Partial(partial) => (
                var.clone(),
                simplify_partial(partial.clone().into_term(), var),
            ),
            _ => (var, value),
        })
        .collect()
}

pub struct Simplifier {
    bindings: Bindings,
    this_var: Symbol,
}

impl Folder for Simplifier {
    fn fold_term(&mut self, t: Term) -> Term {
        let x = self.deref(&t);
        eprintln!("X = {}", x);
        fold_term(x, self)
    }

    fn fold_partial(&mut self, mut p: Partial) -> Partial {
        let mut seen: HashSet<&Operation> = HashSet::new();
        p.constraints = p
            .constraints()
            .iter()
            .filter(|c| *c != &TRUE) // Drop empty constraints.
            .filter(|o| seen.insert(o)) // Deduplicate constraints.
            .cloned()
            .collect();

        if let Some(i) = p.constraints.iter().position(|o| {
            let mut o = o.clone();
            let mut invert = false;
            if o.operator == Operator::Not {
                o = o.args[0].value().as_expression().unwrap().clone();
                invert = true;
            }
            match o.operator {
                Operator::Unify | Operator::Neq => {
                    let left = &o.args[0];
                    let right = &o.args[1];
                    let invert = if invert {
                        o.operator != Operator::Neq
                    } else {
                        o.operator == Operator::Neq
                    };
                    left == right
                        || match (left.value(), right.value()) {
                            (Value::Variable(v), x) if !self.is_this_var(left) && x.is_ground() => {
                                eprintln!("A {} ← {}", left.to_polar(), right.to_polar());
                                self.bind(v.clone(), right.clone(), invert);
                                true
                            }
                            (x, Value::Variable(v))
                                if !self.is_this_var(right) && x.is_ground() =>
                            {
                                eprintln!("B {} ← {}", right.to_polar(), left.to_polar());
                                self.bind(v.clone(), left.clone(), invert);
                                true
                            }
                            (_, Value::Variable(v)) if self.is_this_var(left) => {
                                eprintln!("C {} ← {}", right.to_polar(), left.to_polar());
                                self.bind(v.clone(), left.clone(), invert);
                                false
                            }
                            (Value::Variable(v), _) if self.is_this_var(right) => {
                                eprintln!("D {} ← {}", left.to_polar(), right.to_polar());
                                self.bind(v.clone(), right.clone(), invert);
                                false
                            }
                            _ => false,
                        }
                }
                _ => false,
            }
        }) {
            eprintln!("CHOSEN CONSTRAINT: {}", &p.constraints[i].to_polar());
            p.constraints.remove(i);
        }
        fold_partial(p, self)
    }

    fn fold_operation(&mut self, o: Operation) -> Operation {
        fold_operation(
            match o.operator {
                // Collapse trivial conjunctions & disjunctions.
                Operator::And | Operator::Or if o.args.len() == 1 => o.args[0]
                    .value()
                    .as_expression()
                    .expect("expression")
                    .clone(),

                Operator::Unify
                | Operator::Eq
                | Operator::Neq
                | Operator::Gt
                | Operator::Geq
                | Operator::Lt
                | Operator::Leq => {
                    let left = &o.args[0];
                    let right = &o.args[1];

                    match (left.value().as_expression(), right.value().as_expression()) {
                        (Ok(left), Ok(right))
                            if left.operator == Operator::Not
                                && right.operator == Operator::Not =>
                        {
                            todo!("not 1 = not 2");
                        }
                        (Ok(left), _) if left.operator == Operator::Not => {
                            invert_operation(Operation {
                                operator: o.operator,
                                args: vec![left.args[0].clone(), right.clone()],
                            })
                        }
                        (_, Ok(right)) if right.operator == Operator::Not => {
                            invert_operation(Operation {
                                operator: o.operator,
                                args: vec![left.clone(), right.args[0].clone()],
                            })
                        }
                        _ => o,
                    }
                }
                Operator::Not => match o.args[0].value().as_expression() {
                    Ok(o) => invert_operation(o.clone()),
                    _ => return o,
                },
                _ => o,
            },
            self,
        )
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
    pub fn new(this_var: Symbol) -> Self {
        Self {
            this_var,
            bindings: HashMap::new(),
        }
    }

    pub fn bind(&mut self, var: Symbol, value: Term, invert: bool) {
        // TODO(ap): check that if there's a current value, it's equal to the new one.
        self.bindings.insert(
            var.clone(),
            if invert {
                value.clone_with_value(value!(op!(Not, value.clone())))
            } else {
                value
            },
        );
        eprintln!(
            "Simplifier.bind({}, {}, {})",
            &var,
            self.bindings[&var].to_polar(),
            invert
        );
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

    fn is_this_var(&self, t: &Term) -> bool {
        match t.value() {
            Value::Variable(v) => v == &self.this_var,
            Value::Expression(Operation {
                operator: Operator::Dot,
                args,
            }) => self.is_this_var(&args[0]),
            _ => false,
        }
    }

    /// Substitute `sym!(_"this")` for our variable in a partial.
    fn sub_this(&self, term: Term) -> Term {
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

        fold_term(term, &mut VariableSubber::new(self.this_var.clone()))
    }
}

/// Simplify a partial until quiescence.
fn simplify_partial(mut term: Term, var: Symbol) -> Term {
    let mut simplifier = Simplifier::new(var.clone());
    let mut new;
    loop {
        new = simplifier.fold_term(term.clone());
        eprintln!(
            "SIMPLIFYING {}: {} => {}",
            var,
            term.to_polar(),
            new.to_polar()
        );
        if new == term {
            break;
        }
        term = new;
    }

    // let new = fold_term(new, &mut EmptyAndRemover {});
    simplifier.sub_this(PartialToExpression {}.fold_term(new))
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
