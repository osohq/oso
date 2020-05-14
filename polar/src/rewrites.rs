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

pub fn rewrite_term(term: Term) -> Term {
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

pub fn rewrite_rule(mut rule: Rule) -> Rule {
    // @TODO: make all these id seeds the same one so we don't duplicate vars.
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
        assert_eq!(rewritten.to_polar(), "f(_value_0) := .(a,b,_value_0);");
    }
}

// @TODO: be able to parse the 3 arg . in predicate format as a 3 arg dot.
// parse all 3 arg predicate operators as expressions.
// x + 1 => +(x, 1, ?value)
