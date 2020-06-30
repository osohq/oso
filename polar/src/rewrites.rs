use super::types::*;

/// Replace the left value by the AND of the right and the left
fn and_wrap(a: &mut Term, b: Term) {
    let new_value = Value::Expression(Operation {
        operator: Operator::And,
        args: vec![b, a.clone()],
    });

    a.replace_value(new_value);
}

/// Checks if the expression needs to be rewritten.
/// If so, replaces the value in place with the symbol, and returns the rewritten expression.
fn rewrite(term: &mut Term, kb: &KnowledgeBase) -> Option<Term> {
    match term.value() {
        Value::Expression(Operation {
            operator: Operator::Dot,
            args: lookup_args,
        }) if lookup_args.len() == 2 => {
            let mut lookup_args = lookup_args.clone();
            let symbol = kb.gensym("value");
            let var = Value::Variable(symbol);
            // Take `id` and `offset` from `b` of lookup `a.b`.
            lookup_args.push(lookup_args[1].clone_with_value(var.clone()));
            let lookup = Value::Expression(Operation {
                operator: Operator::Dot,
                args: lookup_args,
            });
            let _ = term.replace_value(var);
            Some(term.clone_with_value(lookup))
        }
        Value::Expression(Operation {
            operator: Operator::New,
            args,
        }) if args.len() == 1 => {
            assert!(matches!(args[0].value(), Value::InstanceLiteral { .. }));
            let symbol = kb.gensym("instance");
            let var = Value::Variable(symbol);
            let result_term = args[0].clone_with_value(var.clone());
            let args = vec![args[0].clone(), result_term];
            let new_op = Value::Expression(Operation {
                operator: Operator::New,
                args,
            });

            term.replace_value(var);
            Some(term.clone_with_value(new_op))
        }
        _ => None,
    }
}

/// Walks the term and does an in-place rewrite
/// Uses `rewrites` as a buffer of new lookup terms
fn do_rewrite(term: &mut Term, kb: &mut KnowledgeBase, rewrites: &mut Vec<Term>) {
    term.map_replace(&mut |term| {
        // First, rewrite this term, maybe returning a lookup
        // lookup gets added to rewrites list
        let mut term = term.clone();
        if let Some(mut lookup) = rewrite(&mut term, kb) {
            // recursively rewrite the lookup term if necesary
            do_rewrite(&mut lookup, kb, rewrites);
            rewrites.push(lookup);
        } else if let Value::Expression(op) = term.value() {
            // Next, if this is an expression, we want to immediately
            // do the recursive rewrite in place
            if matches!(op.operator, Operator::And | Operator::Or | Operator::Not) {
                let args = op
                    .args
                    .iter()
                    .map(|arg| {
                        let mut arg = arg.clone();
                        let mut arg_rewrites = Vec::new();
                        // gather all rewrites
                        do_rewrite(&mut arg, kb, &mut arg_rewrites);
                        // immediately rewrite the arg in place
                        for rewrite in arg_rewrites.drain(..).rev() {
                            and_wrap(&mut arg, rewrite);
                        }
                        arg
                    })
                    .collect();
                return term.clone_with_value(Value::Expression(Operation {
                    operator: op.operator,
                    args,
                }));
            }
        }
        term
    });
}

/// Rewrite the parameter term and return all new lookups as a vec
pub fn rewrite_parameter(parameter: &mut Term, kb: &mut KnowledgeBase) -> Vec<Term> {
    let mut rewrites = vec![];
    do_rewrite(parameter, kb, &mut rewrites);
    rewrites
}

/// Rewrite the term in-place
pub fn rewrite_term(term: &mut Term, kb: &mut KnowledgeBase) {
    let mut rewrites = vec![];

    do_rewrite(term, kb, &mut rewrites);

    // any other leftover rewrites which didn't get handled earlier
    // (this should only happen in queries with a single clause)
    for rewrite in rewrites.into_iter().rev() {
        and_wrap(term, rewrite);
    }
}

pub fn rewrite_rule(rule: &mut Rule, kb: &mut KnowledgeBase) {
    rewrite_term(&mut rule.body, kb);

    let mut new_terms = vec![];

    for param in &mut rule.params {
        if let Some(parameter) = &mut param.parameter {
            let mut rewrites = rewrite_parameter(parameter, kb);
            new_terms.append(&mut rewrites);
        }
    }

    if let Value::Expression(Operation {
        operator: Operator::And,
        ref args,
    }) = &mut rule.body.value()
    {
        let mut args = args.clone();
        args.append(&mut new_terms);
        rule.body.replace_value(Value::Expression(Operation {
            operator: Operator::And,
            args,
        }));
    } else {
        panic!("Rule body isn't an and, something is wrong.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ToPolarString;

    // Re-defined here for convenience
    fn parse_query(src: &str) -> Term {
        crate::parser::parse_query(0, src).unwrap()
    }

    fn parse_rules(src: &str) -> Rules {
        crate::parser::parse_rules(src).unwrap()
    }

    #[test]
    fn rewrite_rules() {
        let mut kb = KnowledgeBase::new();
        let rules = parse_rules("f(a.b);");
        let mut rule = rules[0].clone();
        assert_eq!(rule.to_polar(), "f(a.b);");

        // First rewrite
        rewrite_rule(&mut rule, &mut kb);
        assert_eq!(rule.to_polar(), "f(_value_1) := .(a, b(), _value_1);");

        // Check we can parse the rules back again
        let again = parse_rules(&rule.to_polar());
        let again_rule = again[0].clone();
        assert_eq!(again_rule.to_polar(), rule.to_polar());

        // Call rewrite again
        let mut rewrite_again_rule = again_rule.clone();
        rewrite_rule(&mut rewrite_again_rule, &mut kb);
        assert_eq!(rewrite_again_rule.to_polar(), again_rule.to_polar());

        // Chained lookups
        let rules = parse_rules("f(a.b.c);");
        let mut rule = rules[0].clone();
        assert_eq!(rule.to_polar(), "f(a.b.c);");
        rewrite_rule(&mut rule, &mut kb);
        assert_eq!(
            rule.to_polar(),
            "f(_value_2) := .(a, b(), _value_3), .(_value_3, c(), _value_2);"
        );
    }

    #[test]
    fn rewrite_nested_lookups() {
        let mut kb = KnowledgeBase::new();

        // Lookups with args
        let rules = parse_rules("f(a, c) := a.b(c);");
        let mut rule = rules[0].clone();
        assert_eq!(rule.to_polar(), "f(a, c) := a.b(c);");
        rewrite_rule(&mut rule, &mut kb);
        assert_eq!(
            rule.to_polar(),
            "f(a, c) := .(a, b(c), _value_1), _value_1;"
        );

        // Nested lookups
        let rules = parse_rules("f(a, c, e) := a.b(c.d(e.f));");
        let mut rule = rules[0].clone();
        assert_eq!(rule.to_polar(), "f(a, c, e) := a.b(c.d(e.f));");
        rewrite_rule(&mut rule, &mut kb);
        assert_eq!(
            rule.to_polar(),
            "f(a, c, e) := .(e, f(), _value_4), .(c, d(_value_4), _value_3), .(a, b(_value_3), _value_2), _value_2;"
        );
    }

    #[test]
    fn rewrite_terms() {
        let mut kb = KnowledgeBase::new();
        let mut term = parse_query("x, a.b");
        assert_eq!(term.to_polar(), "x, a.b");
        rewrite_term(&mut term, &mut kb);
        assert_eq!(term.to_polar(), "x, .(a, b(), _value_1), _value_1");

        let mut query = parse_query("f(a.b.c)");
        assert_eq!(query.to_polar(), "f(a.b.c)");
        rewrite_term(&mut query, &mut kb);
        assert_eq!(
            query.to_polar(),
            ".(a, b(), _value_3), .(_value_3, c(), _value_2), f(_value_2)"
        );

        let mut term = parse_query("a.b = 1");
        rewrite_term(&mut term, &mut kb);
        assert_eq!(term.to_polar(), ".(a, b(), _value_4), _value_4 = 1");
        let mut term = parse_query("{x: 1}.x = 1");
        assert_eq!(term.to_polar(), "{x: 1}.x = 1");
        rewrite_term(&mut term, &mut kb);
        assert_eq!(term.to_polar(), ".({x: 1}, x(), _value_5), _value_5 = 1");
    }

    #[test]
    fn rewrite_nested_literal() {
        let mut kb = KnowledgeBase::new();
        let mut term = parse_query("new Foo { x: bar.y }");
        assert_eq!(term.to_polar(), "new Foo{x: bar.y}");
        rewrite_term(&mut term, &mut kb);
        assert_eq!(
            term.to_polar(),
            ".(bar, y(), _value_2), new (Foo{x: _value_2}, _instance_1), _instance_1"
        );

        let mut term = parse_query("f(new Foo { x: bar.y })");
        assert_eq!(term.to_polar(), "f(new Foo{x: bar.y})");
        rewrite_term(&mut term, &mut kb);
        assert_eq!(
            term.to_polar(),
            ".(bar, y(), _value_4), new (Foo{x: _value_4}, _instance_3), f(_instance_3)"
        );
    }

    #[test]
    fn rewrite_class_constructor() {
        let mut kb = KnowledgeBase::new();
        let mut term = parse_query("new Foo{a: 1, b: 2}");
        assert_eq!(term.to_polar(), "new Foo{a: 1, b: 2}");

        rewrite_term(&mut term, &mut kb);
        // @ means external constructor
        assert_eq!(
            term.to_polar(),
            "new (Foo{a: 1, b: 2}, _instance_1), _instance_1"
        );
    }

    #[test]
    fn rewrite_nested_class_constructor() {
        let mut kb = KnowledgeBase::new();
        let mut term = parse_query("new Foo{a: 1, b: new Foo{a: 2, b: 3}}");
        assert_eq!(term.to_polar(), "new Foo{a: 1, b: new Foo{a: 2, b: 3}}");

        rewrite_term(&mut term, &mut kb);
        assert_eq!(
            term.to_polar(),
            "new (Foo{a: 2, b: 3}, _instance_2), new (Foo{a: 1, b: _instance_2}, _instance_1), _instance_1"
        );
    }

    #[test]
    fn rewrite_rules_constructor() {
        let mut kb = KnowledgeBase::new();
        let mut rules = parse_rules("rule_test(new Foo{a: 1, b: 2});");
        assert_eq!(rules[0].to_polar(), "rule_test(new Foo{a: 1, b: 2});");

        rewrite_rule(&mut rules[0], &mut kb);
        assert_eq!(
            rules[0].to_polar(),
            "rule_test(_instance_1) := new (Foo{a: 1, b: 2}, _instance_1);"
        )
    }
}
