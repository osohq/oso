use std::collections::HashSet;

use crate::kb::Bindings;
use crate::terms::{Symbol, Term, Value, Operator};
use crate::formatting::ToPolarString;

pub fn simplify_bindings(mut bindings: Bindings) -> Bindings {
    let root_partials = get_roots(&bindings);

    for root in root_partials.iter() {
        let simplified = simplify_partial(bindings.get(root).unwrap().clone(), &bindings);
        bindings.insert(root.clone(), simplified);
    }

    to_expressions(&mut bindings);
    remove_temporaries(&mut bindings);

    bindings
}

fn simplify_partial(term: Term, bindings: &Bindings) -> Term {
    let term = simplify_partial_variables(term, bindings);
    simplify_unify_partials(term, bindings)
}


fn simplify_partial_variables(term: Term, bindings: &Bindings) -> Term {
    term.cloned_map_replace(&mut |term: &Term| {
        if let Value::Variable(name) = term.value() {
            let value = bindings.get(name);
            value
                .cloned()
                .map(|term| simplify_partial(term, bindings))
                .unwrap_or_else(|| {
                    // NOTE this might be an error
                    term.clone()
                })
        } else {
            term.clone()
        }
    })
}

// Take partial(_this = ?) and output ?.
fn simplify_unify_partials(term: Term, _: &Bindings) -> Term {
    term.cloned_map_replace(&mut |term: &Term| {
        if let Value::Partial(p) = term.value() {
            if p.operations().len() == 1 && p.operations().first().unwrap().operator == Operator::Unify {
                let mut op = p.operations().first().unwrap().clone();
                let right = op.args.pop().unwrap();
                let left = op.args.pop().unwrap();

                match (left.value(), right.value()) {
                    (_, Value::Variable(sym)) if sym.0 == "_this" => {
                        left.clone()
                    },
                    (Value::Variable(sym), _) if sym.0 == "_this" => {
                        right.clone()
                    },
                    _ => term.clone()
                }
            } else {
                term.clone()
            }
        } else {
            term.clone()
        }
    })
}

fn get_roots(bindings: &Bindings) -> HashSet<Symbol> {
    let mut roots = HashSet::new();
    for (symbol, val) in bindings.iter() {
        if !symbol.is_temporary_var() {
            if let Value::Partial(_) = val.value() {
                roots.insert(symbol.clone());
            }
        }
    }

    roots
}

fn to_expressions(bindings: &mut Bindings) {
    let mut new_bindings = Bindings::new();

    for (name, val) in bindings.iter() {
        if let Value::Partial(partial) = val.value() {
            let name = name.clone();
            let partial = partial.clone().as_expression();
            new_bindings.insert(name, partial);
        }
    }

    bindings.extend(new_bindings.into_iter());
}

fn remove_temporaries(bindings: &mut Bindings) {
    let mut remove = HashSet::new();

    for (name, _) in bindings.iter() {
        if name.is_temporary_var() {
            remove.insert(name.clone());
        }
    }

    for name in remove.iter() {
        bindings.remove(name);
    }
}

#[cfg(test)]
mod test {
    use crate::macros::*;

    use super::*;
    use crate::partial::Constraints;

    #[test]
    fn test_variable_subsitution() {
        let mut bindings = Bindings::new();

        let mut constraint = Constraints::new(sym!("a"));
        let bar_partial_term = constraint.lookup(term!("foo"), term!(sym!("_value_1")));
        let mut bar_partial = bar_partial_term.value().clone().partial().unwrap();
        bar_partial.unify(term!(1));

        bindings.insert(sym!("a"), term!(constraint));
        bindings.insert(sym!("_value_1"), term!(bar_partial));

        let bindings = simplify_bindings(bindings);
        let a_term = bindings.get(&sym!("a")).unwrap();

        assert_eq!(a_term.to_polar(), "1 = _this.foo");
    }
}
