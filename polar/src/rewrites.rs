use super::types::*;

/// Replace the left value by the AND of the right and the left
fn and_wrap(a: &mut Value, b: Value) {
    let mut old_a = Value::Symbol(Symbol::new("_"));
    std::mem::swap(a, &mut old_a);
    *a = Value::Expression(Operation {
        operator: Operator::And,
        args: vec![Term::new(b), Term::new(old_a)],
    });
}

/// Checks if the expression needs to be rewritten.
/// If so, replaces the value in place with the symbol, and returns the lookup needed
fn rewrite(value: &mut Value, kb: &KnowledgeBase) -> Option<Value> {
    match value {
        Value::Expression(Operation {
            operator: Operator::Dot,
            args: lookup_args,
        }) if lookup_args.len() == 2 => {
            let mut lookup_args = lookup_args.clone();
            let symbol = kb.gensym("value");
            let var = Value::Symbol(symbol);
            lookup_args.push(Term::new(var.clone()));
            let lookup = Value::Expression(Operation {
                operator: Operator::Dot,
                args: lookup_args,
            });
            *value = var;
            Some(lookup)
        }
        _ => None,
    }
}

pub fn rewrite_specializer(spec: &mut Term, kb: &KnowledgeBase) -> Vec<Term> {
    let mut rewrites = vec![];

    // Walk the tree, replace rewrite terms with symbols and cache up rewrites to be made next pass.
    let mut find_rewrites =
        |term: &mut Term, _index: &TreeIndex, _insert_point: &Option<TreeIndex>| {
            if let Some((lookup, symbol)) = rewrite(term, kb) {
                rewrites.push(lookup);
                *term = symbol;
            }
        };
    let mut index = vec![];
    let insert_point = None;
    walk_indexed(spec, &mut index, &insert_point, &mut find_rewrites);

    rewrites
}

pub fn rewrite_term(term: &mut Term, kb: &KnowledgeBase) {
    let mut rewrites = vec![];

    fn do_rewrite(term: &mut Term, kb: &KnowledgeBase, rewrites: &mut Vec<Value>) {
        term.map_in_place(&mut |value| {
            // First, rewrite this term in place, maybe returning a lookup
            // lookup gets added to rewrites list
            if let Some(lookup) = rewrite(value, kb) {
                rewrites.push(lookup);
            }

            // Next, if this is an expression, we want to immediately
            // do the recursive rewrite in place
            if let Value::Expression(op) = value {
                if matches!(op.operator, Operator::And | Operator::Or | Operator::Not) {
                    for arg in op.args.iter_mut() {
                        let mut arg_rewrites = Vec::new();
                        // gather all rewrites
                        do_rewrite(arg, kb, &mut arg_rewrites);
                        // immediately rewrite the arg in place
                        for rewrite in arg_rewrites.drain(..) {
                            and_wrap(&mut arg.value, rewrite);
                        }
                    }
                }
            }
        });
    };

    do_rewrite(term, kb, &mut rewrites);

    // any other leftover rewrites which didn't get handled earlier
    // (this should only happen in queries with a single clause)
    for rewrite in rewrites {
        and_wrap(&mut term.value, rewrite);
    }
}

pub fn rewrite_rule(rule: &mut Rule, kb: &KnowledgeBase) {
    rewrite_term(&mut rule.body, kb);

    let mut new_terms = vec![];

    for param in &mut rule.params {
        if let Some(specializer) = &mut param.specializer {
            let mut rewrites = rewrite_specializer(specializer, kb);
            new_terms.append(&mut rewrites);
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
        let kb = KnowledgeBase::new();
        let rules = parse_rules("f(a.b);").unwrap();
        let mut rule = rules[0].clone();
        assert_eq!(rule.to_polar(), "f(a.b);");

        // First rewrite
        rewrite_rule(&mut rule, &kb);
        assert_eq!(rule.to_polar(), "f(_value_1) := .(a,b,_value_1);");

        // Check we can parse the rules back again
        let again = parse_rules(&rule.to_polar()).unwrap();
        let again_rule = again[0].clone();
        assert_eq!(again_rule.to_polar(), rule.to_polar());

        // Call rewrite again
        let mut rewrite_again_rule = again_rule.clone();
        rewrite_rule(&mut rewrite_again_rule, &kb);
        assert_eq!(rewrite_again_rule.to_polar(), again_rule.to_polar());
    }
    #[test]
    fn rewrite_terms() {
        let kb = KnowledgeBase::new();

        let term = parse_query("x,a.b").unwrap();
        assert_eq!(term.to_polar(), "x,a.b");
        rewrite_term(term, &kb);
        assert_eq!(term.to_polar(), "x,.(a,b,_value_2),_value_2");

        let query = parse_query("f(a.b.c)").unwrap();
        assert_eq!(query.to_polar(), "f(a.b.c)");
        rewrite_term(query, &kb);
        assert_eq!(
            term.to_polar(),
            ".(a,b,_value_3),.(_value_3,c,_value_4),f(_value_4)"
        );

        let rules = parse_rules("f(a.b.c);").unwrap();
        let rule = rules[0].clone();
        assert_eq!(rule.to_polar(), "f(a.b.c);");
        rewrite_rule(rule, &kb);
        assert_eq!(
            rule.to_polar(),
            "f(_value_6) := .(a,b,_value_5),.(_value_5,c,_value_6);"
        );

        let term = parse_query("a.b = 1").unwrap();
        rewrite_term(term, &kb);
        assert_eq!(term.to_polar(), ".(a,b,_value_7),_value_7=1");
        let term = parse_query("{x: 1}.x = 1").unwrap();
        assert_eq!(term.to_polar(), "{x: 1}.x=1");
        rewrite_term(term, &kb);
        assert_eq!(term.to_polar(), ".({x: 1},x,_value_8),_value_8=1");
    }
}
