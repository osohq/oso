use super::types::*;

/// An index into a term (which is a tree.)
/// The index represents the position of a term in the tree.
/// For every level accross, we increment the current index. And for every level down we add a new index to the list.
/// For keys (in dictionaries and instance literals) we use the key instead of the arg index for the index value.
/// eg if the root is `foo(1, bar({x: 1},3))`
/// the nodes indexes of the nodes are
/// [] foo(1, bar({x: 1},3))
/// [0] 1
/// [1] bar({x: 1},3)
/// [1, 0] {x: 1}
/// [1, 0, x] 1
/// [1, 1] 3
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Index {
    I(usize),
    K(Symbol),
}
pub type TreeIndex = Vec<Index>;

/// Walks the term, keeping track of the tree index and the insertion_point which is the
/// argument of the closest enclosing (and, or, not) expression.
/// This is the place child terms that need to be rewritten would be inserted.
pub fn walk_indexed<F>(
    term: &mut Term,
    index: &mut TreeIndex,
    insert_point: &Option<TreeIndex>,
    f: &mut F,
) where
    F: FnMut(&mut Term, &TreeIndex, &Option<TreeIndex>),
{
    match &mut term.value {
        Value::Integer(_) => (),
        Value::String(_) => (),
        Value::Boolean(_) => (),
        Value::ExternalInstance(_) => (),
        Value::ExternalInstanceLiteral(instance) => {
            for (k, t) in &mut instance.fields.fields.iter_mut() {
                index.push(Index::K(k.clone()));
                walk_indexed(t, index, insert_point, f);
                index.pop();
            }
        }
        Value::InstanceLiteral(instance) => {
            for (k, t) in &mut instance.fields.fields.iter_mut() {
                index.push(Index::K(k.clone()));
                walk_indexed(t, index, insert_point, f);
                index.pop();
            }
        }
        Value::Dictionary(dict) => {
            for (k, t) in &mut dict.fields.iter_mut() {
                index.push(Index::K(k.clone()));
                walk_indexed(t, index, insert_point, f);
                index.pop();
            }
        }
        Value::Call(pred) => {
            for (i, t) in &mut pred.args.iter_mut().enumerate() {
                index.push(Index::I(i));
                walk_indexed(t, index, insert_point, f);
                index.pop();
            }
        }
        Value::List(list) => {
            for (i, t) in &mut list.iter_mut().enumerate() {
                index.push(Index::I(i));
                walk_indexed(t, index, insert_point, f);
                index.pop();
            }
        }
        Value::Symbol(_) => (),
        Value::Expression(op) => {
            let mut is_insert_op = false;
            match op.operator {
                Operator::And | Operator::Or | Operator::Not => {
                    is_insert_op = true;
                }
                _ => (),
            };
            for (i, t) in &mut op.args.iter_mut().enumerate() {
                index.push(Index::I(i));
                if is_insert_op {
                    // If this is an (and, or, not) expression. Then the insertion point for rewritten
                    // expressions will be whichever arg we are traversing.
                    let new_insert_point = Some(index.clone());
                    walk_indexed(t, index, &new_insert_point, f);
                } else {
                    walk_indexed(t, index, insert_point, f);
                }
                index.pop();
            }
        }
    };
    f(term, index, insert_point)
}

/// Takes two terms and wraps them in an AND.
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

/// Checks if the expression needs to be rewritten.
/// If so, returns a tuple of the rewritten expression and the generated symbol to replace it with.
fn rewrite(term: &mut Term, kb: &mut KnowledgeBase) -> Option<(Term, Term)> {
    match &term.value {
        Value::Expression(Operation {
            operator: Operator::Dot,
            args: lookup_args,
        }) if lookup_args.len() == 2 => {
            let mut lookup_args = lookup_args.clone();
            let symbol = kb.gensym("value");
            let var = Term::new(Value::Symbol(symbol));
            lookup_args.push(var.clone());
            let lookup = Term {
                value: Value::Expression(Operation {
                    operator: Operator::Dot,
                    args: lookup_args,
                }),
                id: 0,
                offset: 0,
            };
            Some((lookup, var))
        }
        Value::InstanceLiteral(literal) => {
            let external_id = kb.new_id();
            let external_instance = Term {
                value: Value::ExternalInstance(ExternalInstance {
                    instance_id: external_id,
                }),
                id: 0,
                offset: 0,
            };
            let external_instance_literal = Term {
                value: Value::ExternalInstanceLiteral(literal.clone()),
                id: 0,
                offset: 0,
            };
            let make_args = vec![external_instance_literal, external_instance.clone()];
            let make = Term {
                value: Value::Expression(Operation {
                    operator: Operator::Make,
                    args: make_args,
                }),
                id: 0,
                offset: 0,
            };
            Some((make, external_instance))
        }
        _ => None,
    }
}

pub fn rewrite_term(mut term: Term, kb: &mut KnowledgeBase) -> Term {
    let mut rewrites = vec![];

    // Walk the tree, replace rewrite terms with symbols and cache up rewrites to be made next pass.
    let mut find_rewrites =
        |term: &mut Term, _index: &TreeIndex, insert_point: &Option<TreeIndex>| {
            if let Some((lookup, symbol)) = rewrite(term, kb) {
                if let Some(insert_point) = insert_point {
                    rewrites.push((lookup, insert_point.clone()));
                } else {
                    rewrites.push((lookup, vec![]))
                }
                *term = symbol;
            }
        };
    let mut index = vec![];
    let insert_point = None;
    walk_indexed(&mut term, &mut index, &insert_point, &mut find_rewrites);

    rewrites.reverse();
    let mut do_rewrites =
        |term: &mut Term, index: &TreeIndex, _insert_point: &Option<TreeIndex>| {
            for (lookup, i) in &rewrites {
                if index == i {
                    let new_t = and_wrap(lookup.clone(), term.clone());
                    *term = new_t;
                }
            }
        };
    let mut index = vec![];
    let insert_point = None;
    walk_indexed(&mut term, &mut index, &insert_point, &mut do_rewrites);

    term
}

pub fn rewrite_rule(mut rule: Rule, kb: &mut KnowledgeBase) -> Rule {
    rule.body = rewrite_term(rule.body, kb);

    let mut new_terms = vec![];
    for param in &mut rule.params {
        if let Some((lookup, symbol)) = rewrite(param, kb) {
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
        let mut kb = KnowledgeBase::new();
        let rules = parse_rules("f(a.b);").unwrap();
        let rule = rules[0].clone();
        assert_eq!(rule.to_polar(), "f(a.b);");
        let rewritten = rewrite_rule(rule, &mut kb);
        assert_eq!(rewritten.to_polar(), "f(_value_1) := .(a,b,_value_1);");
        let again = parse_rules(&rewritten.to_polar()).unwrap();
        let again_rule = again[0].clone();
        assert_eq!(again_rule.to_polar(), rewritten.to_polar());
        let again_rewritten = rewrite_rule(again_rule.clone(), &mut kb);
        assert_eq!(again_rewritten.to_polar(), again_rule.to_polar());
        let term = parse_query("x,a.b").unwrap();
        assert_eq!(term.to_polar(), "x,a.b");
        let rewritten = rewrite_term(term, &mut kb);
        assert_eq!(rewritten.to_polar(), "x,.(a,b,_value_2),_value_2");

        let term = parse_query("a.b = 1").unwrap();
        let rewritten = rewrite_term(term, &mut kb);
        assert_eq!(rewritten.to_polar(), ".(a,b,_value_3),_value_3=1");
        let term = parse_query("{x: 1}.x = 1").unwrap();
        assert_eq!(term.to_polar(), "{x: 1}.x=1");
        let rewritten = rewrite_term(term, &mut kb);
        assert_eq!(rewritten.to_polar(), ".({x: 1},x,_value_4),_value_4=1");

        let term = parse_query("!{x: a.b}").unwrap();
        assert_eq!(term.to_polar(), "!{x: a.b}");
        let rewritten = rewrite_term(term, &mut kb);
        assert_eq!(rewritten.to_polar(), "!(.(a,b,_value_5),{x: _value_5})");

        let term = parse_query("Foo{x: 1}").unwrap();
        assert_eq!(term.to_polar(), "Foo{x: 1}");
        let rewritten = rewrite_term(term, &mut kb);
        assert_eq!(rewritten.to_polar(), "make(^Foo{x: 1},^{id: 6}),^{id: 6}");
        // @TODO: Make this reparseable.

        let term = parse_query("Foo{x: 1}.x").unwrap();
        assert_eq!(term.to_polar(), "Foo{x: 1}.x");
        let rewritten = rewrite_term(term, &mut kb);
        assert_eq!(
            rewritten.to_polar(),
            "make(^Foo{x: 1},^{id: 7}),.(^{id: 7},x,_value_8),_value_8"
        );
        let again = parse_query(&rewritten.to_polar()).unwrap();
        assert_eq!(again.to_polar(), rewritten.to_polar(),);
        let rewritten_again = rewrite_term(again, &mut kb);
        assert_eq!(rewritten_again.to_polar(), rewritten.to_polar(),);
    }
}
