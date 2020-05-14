use super::types::*;

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

pub fn rewrite(term: &mut Term, gen: &mut VarGenerator) -> Option<(Term, Term)> {
    if let Value::Expression(Operation {
        operator: Operator::Dot,
        args: lookup_args,
    }) = &term.value
    {
        if lookup_args.len() == 2 {
            let mut lookup_args = lookup_args.clone();
            let symbol = gen.gen_var();
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

pub fn rewrite_term(mut term: Term, gen: &mut VarGenerator) -> Term {
    let mut rewrites = vec![];

    // Walk the tree, replace rewrite terms with symbols and cache up rewrites to be made next pass.
    let mut find_rewrites =
        |term: &mut Term, index: Vec<usize>, insert_point: Option<Vec<usize>>| {
            if let Some((symbol, exp)) = rewrite(term, gen) {
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

pub fn rewrite_rule(mut rule: Rule, gen: &mut VarGenerator) -> Rule {
    // @TODO: make all these id seeds the same one so we don't duplicate vars.
    rule.body = rewrite_term(rule.body, gen);

    let mut new_terms = vec![];
    for param in &mut rule.params {
        if let Some((lookup, symbol)) = rewrite(param, gen) {
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
        panic!("Rule body isn't an and, something is wrong.")
    }
    rule
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::*;
    #[test]
    fn rewrites_test() {
        let mut gen = VarGenerator::new();
        let rules = parse_rules("f(a.b);").unwrap();
        let rule = rules[0].clone();
        assert_eq!(rule.to_polar(), "f(a.b);");
        let rewritten = rewrite_rule(rule, &mut gen);
        assert_eq!(rewritten.to_polar(), "f(_value_0) := .(a,b,_value_0);");
        let again = parse_rules(&rewritten.to_polar()).unwrap();
        let again_rule = again[0].clone();
        assert_eq!(again_rule.to_polar(), rewritten.to_polar());
        let again_rewritten = rewrite_rule(again_rule.clone(), &mut gen);
        assert_eq!(again_rewritten.to_polar(), again_rule.to_polar());
        let term = parse_query("x,a.b").unwrap();
        assert_eq!(term.to_polar(), "x,a.b");
        let rewritten = rewrite_term(term, &mut gen);
        assert_eq!(rewritten.to_polar(), "x,.(a,b,_value_1),_value_1");
    }
}
