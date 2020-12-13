use std::collections::{HashMap, HashSet};

use crate::folder::{fold_operation, fold_partial, fold_term, Folder};
use crate::formatting::ToPolarString;
use crate::kb::Bindings;
// use crate::terms::{Operation, Operator, Symbol, Term, TermList, Value};
use crate::terms::{Operation, Operator, Symbol, Term, Value};

use super::Partial;

/// Simplify the values of the bindings to be returned to the host language.
///
/// - For partials, simplify the constraint expressions.
/// - For non-partials, deep deref.
/// TODO(ap): deep dref.
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
        match t.value() {
            Value::Variable(v) if *v != self.this_var && self.bindings.contains_key(v) => {
                fold_term(self.deref(&t), self)
            }
            _ => fold_term(t, self),
        }
    }

    fn fold_partial(&mut self, p: Partial) -> Partial {
        let mut seen: HashSet<&Operation> = HashSet::new();
        let constraints = p
            .constraints
            .iter()
            .filter(|&c| c != &op!(And)) // Drop empty constraints.
            .filter(|o| seen.insert(o)) // Deduplicate constraints.
            .cloned()
            .collect();
        fold_partial(p.clone_with_constraints(constraints), self)
    }

    fn fold_operation(&mut self, o: Operation) -> Operation {
        if o.operator == Operator::Unify {
            assert_eq!(o.args.len(), 2);
            if o.args[0] == o.args[1] {
                op!(And)
            } else if self.bindings.is_empty() {
                let left = &o.args[0];
                let right = &o.args[1];
                match (left.value(), right.value()) {
                    (Value::Variable(v), x) if x.is_ground() => {
                        eprintln!("A {} ← {}", left.to_polar(), right.to_polar());
                        self.bind(v.clone(), right.clone());
                        op!(And)
                    }
                    (x, Value::Variable(v)) if x.is_ground() => {
                        eprintln!("B {} ← {}", right.to_polar(), left.to_polar());
                        self.bind(v.clone(), left.clone());
                        op!(And)
                    }
                    (Value::Variable(v), Value::Variable(w)) if *v == self.this_var => {
                        eprintln!("C {} ← {}", right.to_polar(), left.to_polar());
                        self.bind(w.clone(), left.clone());
                        fold_operation(o, self)
                    }
                    (Value::Variable(v), Value::Variable(w)) if *w == self.this_var => {
                        eprintln!("D {} ← {}", left.to_polar(), right.to_polar());
                        self.bind(v.clone(), right.clone());
                        fold_operation(o, self)
                    }
                    _ => fold_operation(o, self),
                }
            } else {
                fold_operation(o, self)
            }
        } else {
            fold_operation(o, self)
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
    pub fn new(this_var: Symbol) -> Self {
        Self {
            this_var,
            bindings: HashMap::new(),
        }
    }

    pub fn bind(&mut self, var: Symbol, value: Term) {
        // TODO(ap): check that if there's a current value, it's equal to the new one.
        self.bindings.insert(var, value);
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
            if simplifier.bindings.is_empty()
                || term.value().as_partial().unwrap().constraints.len() == 1
            {
                break;
            } else {
                simplifier.bindings.drain();
            }
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
