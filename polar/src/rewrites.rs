use super::types::*;

/// Replace the left value by the AND of the right and the left
fn and_wrap(a: &mut Term, b: Value) {
    let mut old_a = Value::Symbol(Symbol::new("_"));
    std::mem::swap(&mut a.value, &mut old_a);
    a.value = Value::Expression(Operation {
        operator: Operator::And,
        args: vec![a.clone_with_value(b), a.clone_with_value(old_a)],
    });
}

/// Checks if the expression needs to be rewritten.
/// If so, replaces the value in place with the symbol, and returns the rewritten expression.
fn rewrite(value: &mut Value, kb: &KnowledgeBase) -> Option<Value> {
    match value {
        Value::Expression(Operation {
            operator: Operator::Dot,
            args: lookup_args,
        }) if lookup_args.len() == 2 => {
            let mut lookup_args = lookup_args.clone();
            let symbol = kb.gensym("value");
            let var = Value::Symbol(symbol);
            // Take `id` and `offset` from `b` of lookup `a.b`.
            lookup_args.push(lookup_args[1].clone_with_value(var.clone()));
            let lookup = Value::Expression(Operation {
                operator: Operator::Dot,
                args: lookup_args,
            });
            *value = var;
            Some(lookup)
        }
        Value::Expression(Operation {
            operator: Operator::New,
            args,
        }) if args.len() == 1 => {
            let literal = args[0].clone().value.instance_literal().unwrap();
            let symbol = kb.gensym("instance");
            let var = Value::Symbol(symbol);

            let args = vec![
                Term::new(Value::InstanceLiteral(literal)),
                Term::new(var.clone()),
            ];
            let new_op = Value::Expression(Operation {
                operator: Operator::New,
                args,
            });

            *value = var;
            Some(new_op)
        }
        _ => None,
    }
}

/// Walks the term and does an in-place rewrite
/// Uses `rewrites` as a buffer of new lookup terms
fn do_rewrite(term: &mut Term, kb: &mut KnowledgeBase, rewrites: &mut Vec<Value>, src_id: u64) {
    if term.id == 0 {
        term.id = kb.new_id();
        kb.sources.add_term_source(&term, src_id);
    }
    term.walk_mut(&mut |term| {
        // First, rewrite this term in place, maybe returning a lookup
        // lookup gets added to rewrites list
        if let Some(lookup) = rewrite(&mut term.value, kb) {
            let mut lookup_term = term.clone_with_value(lookup);
            // recursively rewrite the lookup term if necesary
            do_rewrite(&mut lookup_term, kb, rewrites, src_id);
            rewrites.push(lookup_term.value);
        }

        // Next, if this is an expression, we want to immediately
        // do the recursive rewrite in place
        if let Value::Expression(ref mut op) = term.value {
            if matches!(op.operator, Operator::And | Operator::Or | Operator::Not) {
                for arg in op.args.iter_mut() {
                    let mut arg_rewrites = Vec::new();
                    // gather all rewrites
                    do_rewrite(arg, kb, &mut arg_rewrites, src_id);
                    // immediately rewrite the arg in place
                    for rewrite in arg_rewrites.drain(..).rev() {
                        and_wrap(arg, rewrite);
                    }
                }
                return false;
            }
        }
        true
    });
}

/// Rewrite the spec term and return all new lookups as a vec
pub fn rewrite_specializer(spec: &mut Term, kb: &mut KnowledgeBase, src_id: u64) -> Vec<Term> {
    let mut rewrites = vec![];
    do_rewrite(spec, kb, &mut rewrites, src_id);

    rewrites
        .into_iter()
        .map(|value| spec.clone_with_value(value))
        .collect()
}

/// Rewrite the term in-place
pub fn rewrite_term(term: &mut Term, kb: &mut KnowledgeBase, src_id: u64) {
    let mut rewrites = vec![];

    do_rewrite(term, kb, &mut rewrites, src_id);

    // any other leftover rewrites which didn't get handled earlier
    // (this should only happen in queries with a single clause)
    for rewrite in rewrites.into_iter().rev() {
        and_wrap(term, rewrite);
    }
}

pub fn rewrite_rule(rule: &mut Rule, kb: &mut KnowledgeBase, src_id: u64) {
    rewrite_term(&mut rule.body, kb, src_id);

    let mut new_terms = vec![];

    for param in &mut rule.params {
        if let Some(specializer) = &mut param.specializer {
            let mut rewrites = rewrite_specializer(specializer, kb, src_id);
            new_terms.append(&mut rewrites);
        }
    }

    if let Value::Expression(Operation {
        operator: Operator::And,
        ref mut args,
    }) = &mut rule.body.value
    {
        args.append(&mut new_terms);
    } else {
        panic!("Rule body isn't an and, something is wrong.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::*;
    use crate::ToPolarString;
    #[test]
    fn rewrite_rules() {
        let mut kb = KnowledgeBase::new();
        let rules = parse_rules("f(a.b);").unwrap();
        let mut rule = rules[0].clone();
        assert_eq!(rule.to_polar(), "f(a.b);");

        // First rewrite
        rewrite_rule(&mut rule, &mut kb, 0);
        assert_eq!(rule.to_polar(), "f(_value_1) := .(a, b(), _value_1);");

        // Check we can parse the rules back again
        let again = parse_rules(&rule.to_polar()).unwrap();
        let again_rule = again[0].clone();
        assert_eq!(again_rule.to_polar(), rule.to_polar());

        // Call rewrite again
        let mut rewrite_again_rule = again_rule.clone();
        rewrite_rule(&mut rewrite_again_rule, &mut kb, 0);
        assert_eq!(rewrite_again_rule.to_polar(), again_rule.to_polar());

        // Chained lookups
        let rules = parse_rules("f(a.b.c);").unwrap();
        let mut rule = rules[0].clone();
        assert_eq!(rule.to_polar(), "f(a.b.c);");
        rewrite_rule(&mut rule, &mut kb, 0);
        assert_eq!(
            rule.to_polar(),
            "f(_value_2) := .(a, b(), _value_3), .(_value_3, c(), _value_2);"
        );
    }

    #[test]
    fn rewrite_nested_lookups() {
        let mut kb = KnowledgeBase::new();

        // Lookups with args
        let rules = parse_rules("f(a, c) := a.b(c);").unwrap();
        let mut rule = rules[0].clone();
        assert_eq!(rule.to_polar(), "f(a, c) := a.b(c);");
        rewrite_rule(&mut rule, &mut kb, 0);
        assert_eq!(
            rule.to_polar(),
            "f(a, c) := .(a, b(c), _value_1), _value_1;"
        );

        // Nested lookups
        let rules = parse_rules("f(a, c, e) := a.b(c.d(e.f));").unwrap();
        let mut rule = rules[0].clone();
        assert_eq!(rule.to_polar(), "f(a, c, e) := a.b(c.d(e.f));");
        rewrite_rule(&mut rule, &mut kb, 0);
        assert_eq!(
            rule.to_polar(),
            "f(a, c, e) := .(e, f(), _value_4), .(c, d(_value_4), _value_3), .(a, b(_value_3), _value_2), _value_2;"
        );
    }

    #[test]
    fn rewrite_terms() {
        let mut kb = KnowledgeBase::new();
        let mut term = parse_query("x, a.b").unwrap();
        assert_eq!(term.to_polar(), "x, a.b");
        rewrite_term(&mut term, &mut kb, 0);
        assert_eq!(term.to_polar(), "x, .(a, b(), _value_1), _value_1");

        let mut query = parse_query("f(a.b.c)").unwrap();
        assert_eq!(query.to_polar(), "f(a.b.c)");
        rewrite_term(&mut query, &mut kb, 0);
        assert_eq!(
            query.to_polar(),
            ".(a, b(), _value_3), .(_value_3, c(), _value_2), f(_value_2)"
        );

        let mut term = parse_query("a.b = 1").unwrap();
        rewrite_term(&mut term, &mut kb, 0);
        assert_eq!(term.to_polar(), ".(a, b(), _value_4), _value_4 = 1");
        let mut term = parse_query("{x: 1}.x = 1").unwrap();
        assert_eq!(term.to_polar(), "{x: 1}.x = 1");
        rewrite_term(&mut term, &mut kb, 0);
        assert_eq!(term.to_polar(), ".({x: 1}, x(), _value_5), _value_5 = 1");
    }

    #[test]
    fn rewrite_nested_literal() {
        let mut kb = KnowledgeBase::new();
        let mut term = parse_query("new Foo { x: bar.y }").unwrap();
        assert_eq!(term.to_polar(), "new Foo{x: bar.y}");
        rewrite_term(&mut term, &mut kb, 0);
        assert_eq!(term.to_polar(), ".(bar, y(), _value_2), new (Foo{x: _value_2}, _instance_1), _instance_1");

        let mut term = parse_query("f(new Foo { x: bar.y })").unwrap();
        assert_eq!(term.to_polar(), "f(new Foo{x: bar.y})");
        rewrite_term(&mut term, &mut kb, 0);
        assert_eq!(term.to_polar(), ".(bar, y(), _value_4), new (Foo{x: _value_4}, _instance_3), f(_instance_3)");
    }

    #[test]
    fn rewrite_class_constructor() {
        let mut kb = KnowledgeBase::new();
        let mut term = parse_query("new Foo{a: 1, b: 2}").unwrap();
        assert_eq!(term.to_polar(), "new Foo{a: 1, b: 2}");

        rewrite_term(&mut term, &mut kb, 0);
        // @ means external constructor
        assert_eq!(
            term.to_polar(),
            "new (Foo{a: 1, b: 2}, _instance_1), _instance_1"
        );
    }

    #[test]
    fn rewrite_nested_class_constructor() {
        let mut kb = KnowledgeBase::new();
        let mut term = parse_query("new Foo{a: 1, b: new Foo{a: 2, b: 3}}").unwrap();
        assert_eq!(term.to_polar(), "new Foo{a: 1, b: new Foo{a: 2, b: 3}}");

        rewrite_term(&mut term, &mut kb, 0);
        // @ means external constructor
        assert_eq!(
            term.to_polar(),
            "new (Foo{a: 2, b: 3}, _instance_2), new (Foo{a: 1, b: _instance_2}, _instance_1), _instance_1"
        );
    }

    #[test]
    fn rewrite_rules_constructor() {
        let mut kb = KnowledgeBase::new();
        let mut rules = parse_rules("rule_test(new Foo{a: 1, b: 2});").unwrap();
        assert_eq!(rules[0].to_polar(), "rule_test(new Foo{a: 1, b: 2});");

        rewrite_rule(&mut rules[0], &mut kb, 0);
        assert_eq!(
            rules[0].to_polar(),
            "rule_test(_instance_1) := new (Foo{a: 1, b: 2}, _instance_1);"
        )
    }
}
