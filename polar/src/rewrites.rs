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

    let rewrite_term_helper = move |value: &Value| -> Value {
        match value {
            Value::Expression(Operation {
                operator: Operator::Dot,
                mut args,
            }) => {
                let symbol = gen_var(&mut i);
                args.push(symbol.clone());
                let lookup = Term {
                    value: Value::Expression(Operation {
                        operator: Operator::Dot,
                        args,
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
            _ => value.clone(),
        }
    };

    term.map(&mut rewrite_term_helper)
}

pub fn rewrite_rule(rule: Rule) -> Rule {
    let Rule { name, params, body } = rule;
    let body = rewrite_term(body);
    // rewrite params
    Rule { name, params, body }
}
