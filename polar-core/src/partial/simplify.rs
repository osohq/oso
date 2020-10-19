use std::collections::HashSet;

use crate::folder::{fold_constraints, fold_operation, fold_term, Folder};
// use crate::formatting::ToPolarString;
use crate::kb::Bindings;
use crate::partial::Constraints;
use crate::terms::{Operation, Operator, Symbol, Term, TermList, Value};

// Variable(?) <= bound value which might be a partial
//
// Top level unify
//
// a: _this = ?
// ?
//
// Dot op and comparison or unify
//
// a: (_this.foo = _temp)
// _temp: this = ?
//
// a: _this.foo = _temp
// _temp: this > 0
//
// a: _this.foo = _temp
// _temp: this > 0, this = 1, this < 0
//
// _this.foo > 0 and _this.foo = 1 and _this.foo < 0
//
// a: _this.a = _value_2_8
// _value_2_8: _this.b = _value_1_9
// _value_1_9: _this > 0
//
// a: _this.a.b = _value_1_9
// _value_1_9: _this > 0
//
// a: _this.a.b > 0

pub fn simplify_bindings(mut bindings: Bindings) -> Bindings {
    let root_partials = get_roots(&bindings);

    for root in root_partials.iter() {
        let simplified = simplify_partial(bindings.get(root).unwrap().clone());
        bindings.insert(root.clone(), simplified);
    }

    to_expressions(&mut bindings);
    remove_temporaries(&mut bindings);

    bindings
}

pub struct Simplifier;

impl Folder for Simplifier {
    fn fold_term(&mut self, t: Term) -> Term {
        if let Value::Partial(Constraints {
            operations,
            variable,
        }) = t.value()
        {
            let single_unify = operations.len() == 1
                && matches!(operations.first().unwrap().operator, Operator::Unify);

            if single_unify {
                fn sub_this(term: &Term, default: &Term) -> Term {
                    if is_this_arg(term.value()) {
                        default.clone()
                    } else {
                        term.clone()
                    }
                }

                let mut map_ops = |ops: &[Operation], replacement: &Term| -> TermList {
                    ops.iter()
                        .map(|o| Operation {
                            operator: o.operator,
                            args: o.args.iter().map(|a| sub_this(a, replacement)).collect(),
                        })
                        .map(|o| {
                            replacement.clone_with_value(Value::Expression(fold_operation(o, self)))
                        })
                        .collect()
                };

                let unify = operations.first().unwrap();
                let left = unify.args.get(0).unwrap();
                let right = unify.args.get(1).unwrap();
                t.clone_with_value(Value::Expression(Operation {
                    operator: Operator::And,
                    args: match (left.value(), right.value()) {
                        (Value::Partial(c), Value::Expression(_)) => map_ops(&c.operations, right),
                        (Value::Expression(_), Value::Partial(c)) => map_ops(&c.operations, left),
                        (Value::Partial(_), _) => vec![fold_term(right.clone(), self)],
                        (_, Value::Partial(_)) => vec![fold_term(left.clone(), self)],
                        _ => return fold_term(not_this_arg(unify).unwrap(), self),
                    },
                }))
            } else {
                t.clone_with_value(Value::Partial(fold_constraints(
                    Constraints {
                        operations: operations.clone(),
                        variable: variable.clone(),
                    },
                    self,
                )))
            }
        } else {
            fold_term(t, self)
        }
    }
}

#[allow(clippy::let_and_return)]
fn simplify_partial(term: Term) -> Term {
    Simplifier {}.fold_term(term)
}

fn not_this_arg(operation: &Operation) -> Option<Term> {
    let left = operation.args.get(0).unwrap();
    let right = operation.args.get(1).unwrap();

    match (is_this_arg(left.value()), is_this_arg(right.value())) {
        (false, true) => Some(left.clone()),
        (true, false) => Some(right.clone()),
        _ => None,
    }
}

fn is_this_arg(value: &Value) -> bool {
    matches!(value, Value::Variable(sym) if sym.0 == "_this")
}

// partial(_x_5) { partial(_value_1_6) { _this > 0, _this > 1 } = _this.a }

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
            let partial = partial.clone().into_expression();
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
