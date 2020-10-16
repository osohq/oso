use std::collections::HashSet;

//use crate::formatting::ToPolarString;
use crate::folder::Folder;
use crate::kb::Bindings;
use crate::partial::Constraints;
use crate::terms::{Operation, Operator, Symbol, Term, Value};

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
    fn fold_constraints(&mut self, c: Constraints) -> Constraints {
        c
    }
}

#[allow(clippy::let_and_return)]
fn simplify_partial(term: Term) -> Term {
    Simplifier {}.fold_term(term)
}

fn dot_field(op: &Value) -> usize {
    match op {
        Value::Expression(op) if op.operator == Operator::Dot => 1,
        Value::Partial(_) => 2,
        _ => 0,
    }
}

// fn simplify_dot_ops(term: Term, bindings: &Bindings) -> Term {
//
//
//
//
//     // folder only cares about Expression(Operation { operator: Dot | Unify })
//
//     if let Value::Partial(partial) = term.value() {
//
//     term#<{(|.cloned_map_replace(&mut |term: &Term| {
//         if let Value::Partial(partial) = term.value() {
//             let mut operations = vec![];
//             for operation in partial.operations() {
//                 if operation.args.len() == 2 {
//                     let left = operation.args.get(0).unwrap().value().clone();
//                     let right = operation.args.get(1).unwrap().value().clone();
//                     match (dot_field(&left), dot_field(&right)) {
//                         (1, 2) => {
//                             let right = simplify_dot_ops(term!(right), bindings);
//                             simplify_dot_ops_helper(&left, right.value(), &mut operations, bindings)
//                         }
//                         (2, 1) => {
//                             let left = simplify_dot_ops(term!(left), bindings);
//                             simplify_dot_ops_helper(&right, left.value(), &mut operations, bindings)
//                         }
//                         (_, _) => operations.push(operation.clone()),
//                     };
//                 } else {
//                     operations.push(operation.clone());
//                 }
//             }
//
//             //eprintln!("ops: {:?}", operations.iter().map(|op| op.to_polar()).collect::<Vec<String>>());
//             term.clone_with_value(Value::Partial(partial.clone_with_operations(operations)))
//         } else {
//             term.clone()
//         }
//     })|)}>#
// }

fn simplify_dot_ops_helper(
    dot_op: &Value,
    partial: &Value,
    operations: &mut Vec<Operation>,
    _: &Bindings,
) {
    //eprintln!("dot_op: {:?}", &dot_op.to_polar());
    //eprintln!("other: {:?}", &other.to_polar());
    if let Value::Partial(partial) = partial {
        // TODO: This transformation doesn't work for nested dots.
        let mut args = vec![];
        for operation in partial.operations() {
            //eprintln!("op: {:?}\nargs: {:?}", operation.operator, operation.args.iter().map(|op| op.to_polar()).collect::<Vec<String>>());
            if operation.args.len() == 2 {
                let left = operation.args.get(0).unwrap().value();
                let right = operation.args.get(1).unwrap().value();
                match (is_this_arg(left), is_this_arg(right)) {
                    (true, false) => {
                        args.push(term!(Operation {
                            operator: operation.operator,
                            args: vec![term!(dot_op.clone()), term!(right.clone())]
                        }));
                    }
                    (false, true) => {
                        args.push(term!(Operation {
                            operator: operation.operator,
                            args: vec![term!(left.clone()), term!(dot_op.clone())]
                        }));
                    }
                    (_, _) => panic!("invalid"),
                }
            } else {
                args.push(term!(operation.clone()))
            }
        }

        operations.push(Operation {
            operator: Operator::And,
            args,
        });
    }
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

// Take partial(_this = ?) and output ?.
fn simplify_unify_partials(term: Term, _: &Bindings) -> Term {
    if let Value::Partial(p) = term.value() {
        let operator = p.operations().first().unwrap().operator;
        let is_unify = matches!(operator, Operator::Unify);

        if p.operations().len() == 1 && is_unify {
            let op = p.operations().first().unwrap();

            match not_this_arg(op) {
                Some(term) => term,
                None => term.clone(),
            }
        } else {
            term.clone()
        }
    } else {
        term.clone()
    }
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
