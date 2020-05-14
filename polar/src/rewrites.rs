use super::types::*;

//pub fn rewrite(term: Term) -> Term {
// if you see a . expression, replace it with a temp symbol?.
// insert the . expression, with a third argument (the temp symbol) in the nearest enclosing AND/OR/NOT expression.
// if the expression isn't in an AND/OR/NOT we have to wrap it in an empty AND.

// this is only for rules
// foo(a.b) => foo(?temp) := .(a,b,?temp);
// foo(a.b) := hi;
// => foo(?temp) := .(a,b,?temp), hi;

// for queries
// foo(a.b) => .(a,b,?x), foo(?x)
// foo(a.b(i)) => .(a,b(i),?x), foo(?x)
//

// a.b(c,d,e) => .(a,b(c,d,e))
// a,b(c,d,e)

// the two cases
// rewrite the head of a rule,
// rewrite (a query or the body of a rule)
//}

// @NOTE(steve): Not sure how to get map to keep context like this so just doing it manually.
// We walk the tree and keep track of the rewrite_point which is the argument of the nearest parent
// And, Or or Not expression that we're traversing.
// We need to reset that arg with an and of our new thing and the origional arg.

// So, you can't keep a pointer to a thing up the tree as you go down it, the lifetimes don't work out.
// We can do it with unsafe pointers, but nobody wants that so I'll just do it with tree indexes.
// This is slightly slower because we have to traverse the tree multiple times but we can use the existing map.

// @NOTE bad
// pub fn walk_rewrite_term(term: &mut Term) {
//     fn walk(term: &mut Term, insert_point: Option<&mut Term>, insert_arg: bool) {
//         {
//             let mut child_insert_point = insert_point;
//             if insert_arg {
//                 child_insert_point = Some(term);
//             }
//             match &mut term.value {
//                 Value::Integer(i) => (),
//                 Value::String(s) => (),
//                 Value::Boolean(b) => (),
//                 Value::ExternalInstance(external_instance) => (),
//                 Value::InstanceLiteral(instance) => (),
//                 Value::Dictionary(dict) => (),
//                 Value::Call(pred) => {
//                     for t in &mut pred.args {
//                         walk(t, child_insert_point, false);
//                     }
//                 }
//                 Value::List(list) => {
//                     for t in list {
//                         walk(t, child_insert_point, false);
//                     }
//                 }
//                 Value::Symbol(sym) => (),
//                 Value::Expression(op) => {
//                     let mut insert_op = false;
//                     match op.operator {
//                         Operator::And | Operator::Or | Operator::Not => {
//                             insert_op = true;
//                         }
//                         _ => (),
//                     };
//                     for t in &mut op.args {
//                         walk(t, child_insert_point, insert_op);
//                     }
//                 }
//             };
//         }
//         if let Some((symbol, exp)) = rewrite(term) {
//             if let Some(insert_point) = insert_point {
//                 let wrapped = and_wrap(exp, insert_point.clone());
//                 *insert_point = wrapped;
//                 *term = symbol;
//             } else {
//                 *term = and_wrap(exp, symbol);
//             }
//         }
//     };
//     walk(term, None, false);
// }

pub fn rewrite_term_old(term: Term) -> Term {
    return term;
    // walk the term
    // if you hit a . expression
    //     generate a new symbol
    //     replace it with the symbol
    //     push a 3 dot expression into the nearest enclosing and
    //     or if there's an or or not, we add a new and of it and the origional arg.
    let mut i = 0;
    fn gen_var(i: &mut i32) -> Term {
        let t = Term {
            id: 0,
            offset: 0,
            value: Value::Symbol(Symbol(format!("_value_{}", i))),
        };
        *i += 1;
        t
    }

    let mut enclosing_term: Option<&mut Operation> = None;
    let enclosing_arg: usize = 0;

    let mut rewrite_term_helper = move |value: &Value| -> Value {
        match value {
            Value::Expression(Operation {
                operator: Operator::Dot,
                args: lookup_args,
            }) => {
                let mut lookup_args = lookup_args.clone();
                let symbol = gen_var(&mut i);
                lookup_args.push(symbol.clone());
                let lookup = Term {
                    value: Value::Expression(Operation {
                        operator: Operator::Dot,
                        args: lookup_args,
                    }),
                    id: 0,
                    offset: 0,
                };

                if let Some(enclosing_term) = &mut enclosing_term {
                    let and_wrap = Term {
                        value: Value::Expression(Operation {
                            operator: Operator::And,
                            args: vec![lookup, enclosing_term.args[enclosing_arg].clone()],
                        }),
                        id: 0,
                        offset: 0,
                    };
                    enclosing_term.args[enclosing_arg] = and_wrap;
                    symbol.value
                } else {
                    Value::Expression(Operation {
                        operator: Operator::And,
                        args: vec![lookup, symbol],
                    })
                }
            }
            // keep walking and keep track of enclosing term.
            _ => value.clone(),
        }
    };

    // gotta walk the tree to know which arg to be.

    term.map(&mut rewrite_term_helper)
}

// Walks the term, keeping track of the tree index and the insertion_point which is the
// closest (and, or, not) expression parent's argument that we're traversing.
// This is the place child terms that need to be rewritten would be inserted.
pub fn walk_indexed<F>(
    term: &mut Term,
    index: Vec<usize>,
    insert_point: Option<Vec<usize>>,
    f: &mut F,
) where
    F: FnMut(&mut Term, Vec<usize>, Option<Vec<usize>>),
{
    match &mut term.value {
        Value::Integer(i) => (),
        Value::String(s) => (),
        Value::Boolean(b) => (),
        Value::ExternalInstance(external_instance) => (),
        Value::InstanceLiteral(instance) => (),
        Value::Dictionary(dict) => (),
        Value::Call(pred) => {
            let mut index = index.clone();
            for (i, t) in &mut pred.args.iter_mut().enumerate() {
                index.push(i);
                walk_indexed(t, index.clone(), insert_point.clone(), f);
                index.pop();
            }
        }
        Value::List(list) => {
            let mut index = index.clone();
            for (i, t) in &mut list.iter_mut().enumerate() {
                index.push(i);
                walk_indexed(t, index.clone(), insert_point.clone(), f);
                index.pop();
            }
        }
        Value::Symbol(sym) => (),
        Value::Expression(op) => {
            let mut is_insert_op = false;
            match op.operator {
                Operator::And | Operator::Or | Operator::Not => {
                    is_insert_op = true;
                }
                _ => (),
            };
            let mut index = index.clone();
            for (i, t) in &mut op.args.iter_mut().enumerate() {
                index.push(i);
                if is_insert_op {
                    walk_indexed(t, index.clone(), Some(index.clone()), f);
                } else {
                    walk_indexed(t, index.clone(), insert_point.clone(), f);
                }
                index.pop();
            }
        }
    };
    f(term, index, insert_point)
}

fn and_wrap(a: Term, b: Term) -> Term {
    Term {
        value: Value::Expression(Operation {
            operator: Operator::And,
            args: vec![a, b],
        }),
        id: 0,
        offset: 0,
    }
}

pub fn rewrite(term: &mut Term) -> Option<(Term, Term)> {
    let mut i = 0;
    fn gen_var(i: &mut i32) -> Term {
        let t = Term {
            id: 0,
            offset: 0,
            value: Value::Symbol(Symbol(format!("_value_{}", i))),
        };
        *i += 1;
        t
    }

    if let Value::Expression(Operation {
        operator: Operator::Dot,
        args: lookup_args,
    }) = &term.value
    {
        if lookup_args.len() == 2 {
            let mut lookup_args = lookup_args.clone();
            let symbol = gen_var(&mut i);
            lookup_args.push(symbol.clone());
            let lookup = Term {
                value: Value::Expression(Operation {
                    operator: Operator::Dot,
                    args: lookup_args,
                }),
                id: 0,
                offset: 0,
            };
            return Some((lookup, symbol));
        }
    }
    None
}

pub fn rewrite_term(mut term: Term) -> Term {
    let mut rewrites = vec![];

    // Walk the tree, replace rewrite terms with symbols and cache up rewrites to be made next pass.
    let mut find_rewrites =
        |term: &mut Term, index: Vec<usize>, insert_point: Option<Vec<usize>>| {
            if let Some((symbol, exp)) = rewrite(term) {
                if let Some(insert_point) = insert_point {
                    rewrites.push((exp, insert_point));
                } else {
                    rewrites.push((exp, index))
                }
                *term = symbol;
            }
            //eprintln!("{:?} {}", index, term.to_polar());
        };
    walk_indexed(&mut term, vec![], None, &mut find_rewrites);

    let mut do_rewrites = |term: &mut Term, index: Vec<usize>, insert_point: Option<Vec<usize>>| {
        for (t, i) in &rewrites {
            if index == *i {
                let new_t = and_wrap(term.clone(), t.clone());
                *term = new_t;
                break;
            }
        }
    };
    walk_indexed(&mut term, vec![], None, &mut do_rewrites);

    term
}

pub fn rewrite_rule(mut rule: Rule) -> Rule {
    // @TODO: make all these id seeds the same one so we don't duplicate vars.
    rule.body = rewrite_term(rule.body);
    let mut i = 0;
    fn gen_var(i: &mut i32) -> Term {
        let t = Term {
            id: 0,
            offset: 0,
            value: Value::Symbol(Symbol(format!("_value_{}", i))),
        };
        *i += 1;
        t
    }

    let mut new_terms = vec![];
    for param in &mut rule.params {
        // Generate a symbol.
        // Rewrite to 3 arg . with symbol.
        // Replace param with symbol.
        // Push 3 arg . as first thing in the body.
        if let Value::Expression(Operation {
            operator: Operator::Dot,
            args: lookup_args,
        }) = &param.value
        {
            if lookup_args.len() == 2 {
                let mut lookup_args = lookup_args.clone();
                let symbol = gen_var(&mut i);
                lookup_args.push(symbol.clone());
                let lookup = Term {
                    value: Value::Expression(Operation {
                        operator: Operator::Dot,
                        args: lookup_args,
                    }),
                    id: 0,
                    offset: 0,
                };
                new_terms.push(lookup);
                *param = symbol;
            }
        }
    }
    if let Value::Expression(Operation {
        operator: Operator::And,
        ref mut args,
    }) = &mut rule.body.value
    {
        new_terms.append(args);
        args.append(&mut new_terms);
    } else {
        panic!("wtf is this?")
    }
    rule
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::*;
    #[test]
    fn rewrites_test() {
        let rules = parse_rules("f(a.b);").unwrap();
        let rule = rules[0].clone();
        assert_eq!(rule.to_polar(), "f(a.b);");
        let rewritten = rewrite_rule(rule);
        /* assert_eq!(rewritten.to_polar(), "f(_value_0) := .(a,b,_value_0);");
        let again = parse_rules(&rewritten.to_polar()).unwrap();
        let again_rule = again[0].clone();
        assert_eq!(again_rule.to_polar(), rewritten.to_polar());
        let again_rewritten = rewrite_rule(again_rule.clone());
        assert_eq!(again_rewritten.to_polar(), again_rule.to_polar()); */

        let term = parse_query("x,a.b").unwrap();
        assert_eq!(term.to_polar(), "x,a.b");
        let rewritten = rewrite_term(term);
        assert_eq!(rewritten.to_polar(), "x,(.(a,b,_value_0),_value_0)");
    }
}

// @TODO: be able to parse the 3 arg . in predicate format as a 3 arg dot.
// parse all 3 arg predicate operators as expressions.
// x + 1 => +(x, 1, ?value)
