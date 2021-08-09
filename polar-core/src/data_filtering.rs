use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::counter::Counter;
use crate::error::{PolarError, PolarResult};
use crate::events::ResultEvent;
use crate::kb::Bindings;
use crate::terms::*;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum Type {
    Base {
        class_tag: String,
    },
    Relationship {
        kind: String,
        other_class_tag: String,
        my_field: String,
        other_field: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct FetchResult {
    id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct Attrib {
    field: String,
    of: FetchResult,
}

// @NOTE(steve): Constraint is sort of an overloaded word.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum Constraint {
    Eq {
        field: String,
        // @NOTE:(steve) I don't really want this to be Term. I want to make sure it's not a constraint
        // or a variable but just a ground value. Wish we had a type for that.
        value: Term,
    },
    In {
        field: String,
        value: Attrib,
    },
}

// The list of constraints passed to a fetching function.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct FetchRequest {
    class_tag: String,
    constraints: Vec<Constraint>,
}

// A Set of fetch requests that may depend on the results of other fetches.
// resolve_order is the order to resolve the fetches in.
// result_id says which result to return.
// @Q(steve): Is it always the last one in the resolve_order?
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct ResultSet {
    requests: HashMap<String, FetchRequest>,
    resolve_order: Vec<String>,
    result_id: String,
}

// @TODO(steve): There is probably more structure than just a union of ResultSets
// I think when we add OR constraints that this will be more of a tree.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct FilterPlan {
    result_sets: Vec<ResultSet>,
}

pub type Types = HashMap<String, HashMap<String, Type>>;
pub type PartialResults = Vec<ResultEvent>;

struct VarInfo {
    cycles: Vec<(Symbol, Symbol)>,
    types: Vec<(Symbol, String)>,
    values: Vec<(Symbol, Term)>,
    relationships: Vec<(Symbol, String, Symbol)>,
}

// @TODO(steve): Better way to handle these checks than just unwraps and asserts.

fn process_result(exp: &Operation) -> VarInfo {
    let mut var_info = VarInfo {
        cycles: vec![],
        types: vec![],
        values: vec![],
        relationships: vec![],
    };
    process_exp(&mut var_info, exp);
    var_info
}

fn dot_var(var_info: &mut VarInfo, var: Term, field: &Term) -> Symbol {
            // TODO(steve): There's a potential name clash here which would be bad. Works for now.
            // but should probably generate this var better.
            let sym = var.value().as_symbol().unwrap();
            let field_str = field.value().as_string().unwrap();
            let new_var = Symbol::new(&format!("{}_dot_{}", sym.0, field_str));

            // Record the relationship between the vars.
            var_info
                .relationships
                .push((sym.clone(), field_str.to_string(), new_var.clone()));
            new_var

}
fn process_exp(var_info: &mut VarInfo, exp: &Operation) -> Option<Term> {
    match exp.operator {
        Operator::And => {
            for arg in &exp.args {
                let inner_exp = arg.value().as_expression().unwrap();
                process_exp(var_info, inner_exp);
            }
        }
        Operator::Dot => {
            // Dot operations return a var that can be unified with.
            // We create a new var to represent the result of the operation.
            assert_eq!(exp.args.len(), 2);
            let mut var = exp.args[0].clone();
            if let Ok(inner_exp) = var.value().as_expression() {
                assert_eq!(inner_exp.operator, Operator::Dot);
                var = process_exp(var_info, inner_exp).unwrap();
            }
            let field = &exp.args[1];
            let new_var = dot_var(var_info, var, field);
            // Return the var so we can unify with it.
            return Some(Term::new_temporary(Value::Variable(new_var)));
        }
        Operator::Isa => {
            assert_eq!(exp.args.len(), 2);
            let lhs = &exp.args[0];
            let rhs = &exp.args[1];
//            println!("dfisa {:?} {:?}", lhs, rhs);
//            let var = lhs.value().as_symbol().unwrap();
//            let pattern = rhs.value().as_pattern().unwrap();
            if let Value::Pattern(Pattern::Instance(InstanceLiteral { tag, fields })) = rhs.value() {
                // @TODO(steve): Handle specializer fields.
                assert!(fields.fields.is_empty());
                let var = match lhs.value() {
//                    Value::Expression(Operation { operator: Operator::Dot, args }) =>

                    Value::Variable(var) | Value::RestVariable(var) => var.clone(),
                    Value::Expression(op) if op.operator == Operator::Dot => dot_var(var_info, op.args[0].clone(), &op.args[1]),
                    _ => todo!(),
                };
                var_info.types.push((var, tag.clone().0))

            } else {
                todo!()
            }
        }
        Operator::Unify => {
            assert_eq!(exp.args.len(), 2);

            let mut lhs = exp.args[0].clone();
            if let Value::Expression(op) = lhs.value() {
                lhs = process_exp(var_info, op).unwrap();
            };

            let mut rhs = exp.args[1].clone();
            if let Value::Expression(op) = rhs.value() {
                rhs = process_exp(var_info, op).unwrap();
            };

            match (lhs.value(), rhs.value()) {
                // Unifying two variables
                (Value::Variable(l), Value::Variable(r)) => {
                    var_info.cycles.push((l.clone(), r.clone()));
                }
                // Unifying a variable with a value
                (Value::Variable(var), val) | (val, Value::Variable(var)) => var_info
                    .values
                    .push((var.clone(), Term::new_temporary(val.clone()))),
                // Unifying something else, I think would be an error in most cases???
                // 1 = 1 is irrelevant for data filtering, other stuff seems like an error.
                (a, b) => {
                    eprintln!("Bad unify: {} = {}", a.to_polar(), b.to_polar());
                    todo!()
                }
            };
        }
        _ => todo!("need a few more of these"),
    }
    None
}

struct Vars {
    variables: HashMap<String, HashSet<Symbol>>,
    relationships: Vec<(String, String, String)>,
    values: HashMap<String, Term>,
    types: HashMap<String, String>,
    this_id: String,
}

fn collapse_vars(var_info: VarInfo) -> Vars {
    // Merge variable cycles.
    let mut joined_cycles: Vec<HashSet<Symbol>> = vec![];
    'cycles: for (l, r) in var_info.cycles {
        // See if we can add to an existing cycle.
        for joined_cycle in &mut joined_cycles {
            if joined_cycle.contains(&l) || joined_cycle.contains(&r) {
                joined_cycle.insert(l);
                joined_cycle.insert(r);
                continue 'cycles;
            }
        }
        // Create new cycle if we couldn't
        let mut new_cycle = HashSet::new();
        new_cycle.insert(l);
        new_cycle.insert(r);
        joined_cycles.push(new_cycle);
    }

    let mut next_id = 0;
    let mut get_id = move || {
        let id = next_id;
        next_id += 1;
        format!("{}", id)
    };

    // Give each cycle an id
    let mut variables: HashMap<String, HashSet<Symbol>> = HashMap::new();
    for joined_cycle in joined_cycles {
        let id = get_id();
        variables.insert(id, joined_cycle);
    }

    // Substitute in relationships
    let mut parent_ids = vec![];
    'relationships: for (parent, _, _) in &var_info.relationships {
        for (id, set) in &mut variables {
            if set.contains(parent) {
                parent_ids.push(id.clone());
                continue 'relationships;
            }
        }
        // Create a new set if we didn't find one.
        let new_id = get_id();
        let mut new_set = HashSet::new();
        new_set.insert(parent.clone());
        variables.insert(new_id.clone(), new_set);
        parent_ids.push(new_id);
    }
    let mut child_ids = vec![];
    'relationships: for (_, _, child) in &var_info.relationships {
        for (id, set) in &mut variables {
            if set.contains(child) {
                child_ids.push(id.clone());
                continue 'relationships;
            }
        }
        // Create a new set if we didn't find one.
        let new_id = get_id();
        let mut new_set = HashSet::new();
        new_set.insert(child.clone());
        variables.insert(new_id.clone(), new_set);
        child_ids.push(new_id);
    }

    // If a.b = c and a.b = d, that means c = d.
    // @Sorry(steve): Wow, what a loop.
    let mut new_unifies = vec![];
    for (i, ((parent_id1, child_id1), (_, field1, _))) in parent_ids
        .iter()
        .zip(child_ids.iter())
        .zip(var_info.relationships.iter())
        .enumerate()
    {
        for (j, ((parent_id2, child_id2), (_, field2, _))) in parent_ids
            .iter()
            .zip(child_ids.iter())
            .zip(var_info.relationships.iter())
            .enumerate()
        {
            if i != j && parent_id1 == parent_id2 && field1 == field2 {
                // Unify children
                new_unifies.push((parent_id1.clone(), parent_id2.clone()));
            }
        }
    }

    // @TODO(steve): There are absolutely bugs in here.
    // If we're turning 0 into 1 and then 0 into 2 it'll just blow up
    // not correctly turn 0 and 1 into 2. Needs some tests.
    for (x, y) in &new_unifies {
        if (x != y) {
            println!("{} ({}, {})", line!(), x, y);
            let mut xs = variables.remove(x).unwrap();
            let ys = variables.remove(y).unwrap();
            xs.extend(ys);
            variables.insert(x.clone(), xs);
        }
    }

    // Substitute in relationship ids.
    // @Sorry(steve): This is a real mess too.
    let mut relationships = vec![];
    for (parent, field, child) in &var_info.relationships {
        let mut parent_id = String::new();
        let mut child_id = String::new();
        for (id, set) in &mut variables {
            if set.contains(parent) {
                parent_id = id.clone();
            }
            if set.contains(child) {
                child_id = id.clone();
            }
        }
        assert_ne!(parent_id, String::new());
        assert_ne!(child_id, String::new());
        relationships.push((parent_id, field.clone(), child_id));
    }

    // I think a var can only have one value since we make sure there's a var for the dot lookup,
    // and if they had aliases they'd be collapsed by now, so it should be an error
    // if foo.name = "steve" and foo.name = "gabe".
    // TODO(steve): How are we going to handle "in"
    let mut values = HashMap::new();
    'values: for (var, value) in var_info.values {
        for (id, set) in &mut variables {
            if set.contains(&var) {
                // @TODO(steve): If we already have a value for it make sure they match don't just
                // overwrite it.
                values.insert(id.clone(), value);
                continue 'values;
            }
        }
        // Create new variable if we didn't find one.
        let new_id = get_id();
        let mut new_set = HashSet::new();
        new_set.insert(var.clone());
        variables.insert(new_id.clone(), new_set);
        values.insert(new_id, value);
    }

    let mut types = HashMap::new();
    'types: for (var, typ) in var_info.types {
        for (id, set) in &mut variables {
            if set.contains(&var) {
                // @TODO(steve): If we already have a type for it make sure they match don't just
                // overwrite it.
                types.insert(id.clone(), typ);
                continue 'types;
            }
        }
        // Create new variable if we didn't find one.
        let new_id = get_id();
        let mut new_set = HashSet::new();
        new_set.insert(var.clone());
        variables.insert(new_id.clone(), new_set);
        types.insert(new_id, typ);
    }

    let mut this_id = String::new();
    for (id, set) in &variables {
        if set.contains(&Symbol::new("_this")) {
            this_id = id.clone()
        }
    }

    Vars {
        variables,
        relationships,
        values,
        types,
        this_id,
    }
}

fn constrain_vars(types: &Types, vars: &Vars, this_type: &str) -> ResultSet {
    let mut result_set = ResultSet {
        requests: HashMap::new(),
        resolve_order: vec![],
        result_id: vars.this_id.clone(),
    };
    constrain_var(&mut result_set, &types, &vars, &vars.this_id, this_type);
    result_set
}

fn constrain_var(
    result_set: &mut ResultSet,
    types: &Types,
    vars: &Vars,
    var_id: &str,
    var_type: &str,
) {
    // @TODO(steve): Probably should check the type against the var types. I think???

    let mut type_def = HashMap::new();
    for (cls, cls_type_def) in types {
        if cls == var_type {
            type_def = cls_type_def.clone();
            break;
        }
    }

    let mut request = if result_set.requests.contains_key(var_id) {
        result_set.requests.remove(var_id).unwrap()
    } else {
        FetchRequest {
            class_tag: var_type.to_string(),
            constraints: vec![],
        }
    };

    for (parent, field, child) in &vars.relationships {
        if parent == var_id {
            if let Some(typ) = type_def.get(field) {
                if let Type::Relationship {
                    kind,
                    other_class_tag,
                    my_field,
                    other_field,
                } = typ
                {
                    constrain_var(result_set, types, vars, child, other_class_tag);
                    request.constraints.push(Constraint::In {
                        field: my_field.clone(),
                        value: Attrib {
                            field: other_field.clone(),
                            of: FetchResult { id: child.clone() },
                        },
                    });
                    continue;
                }
            }
            // Non relationship or unknown type info.
            // @TODO: Handle "in"
            if let Some(value) = vars.values.get(child) {
                request.constraints.push(Constraint::Eq {
                    field: field.clone(),
                    value: value.clone(),
                });
            } else {
                panic!("why?")
            }
        }
    }

    result_set.requests.insert(var_id.to_string(), request);
    result_set.resolve_order.push(var_id.to_string());
}

pub fn build_filter_plan(
    types: Types,
    partial_results: PartialResults,
    variable: &str,
    class_tag: &str,
) -> PolarResult<FilterPlan> {
    // let polar_version = partial_results[0]
    //     .bindings
    //     .get(&Symbol::new("resource"))
    //     .unwrap()
    //     .to_polar();
    // eprintln!("{}", polar_version);
    //
    // let mut requests = HashMap::new();
    // requests.insert(
    //     "0".to_string(),
    //     FetchRequest {
    //         class_tag: "Foo".to_string(),
    //         constraints: vec![Constraint::Eq {
    //             field: "is_fooey".to_string(),
    //             value: Term::new_temporary(Value::Boolean(true)),
    //         }],
    //     },
    // );

    let mut filter_plan = FilterPlan {
        result_sets: vec![],
    };

    // @NOTE(steve): For now we build a ResultSet for each result. Then we put them into a filterplan
    // which effectively means the results should all be UNION'd together.
    // I suspect this structure will change a little bit once we introduce OR.
    for result in partial_results {
        let term = result.bindings.get(&Symbol::new(variable)).unwrap();
//        println!("BFP {}", term.to_polar());
        let exp = term.value().as_expression()?;
        assert_eq!(exp.operator, Operator::And);

        let var_info = process_result(exp);
        let vars = collapse_vars(var_info);
        let result_set = constrain_vars(&types, &vars, class_tag);
        filter_plan.result_sets.push(result_set);
    }

    Ok(filter_plan)
}

// [
// FilterPlan(
// data_sets={
// 0: Constraints(
// cls="Foo",
// constraints=[Constraint(kind="Eq", field="is_fooey", value=True)],
// )
// },
// resolve_order=[0],
// result_set=0,
// )
// ]

mod tests {
    use super::*;

    #[test]
    fn test_serialize() {
        let mut types = HashMap::new();

        let mut foo_types = HashMap::new();
        foo_types.insert(
            "bar_name",
            Type::Base {
                class_tag: "String".to_owned(),
            },
        );
        foo_types.insert(
            "bar",
            Type::Relationship {
                kind: "parent".to_owned(),
                other_class_tag: "Bar".to_owned(),
                my_field: "bar_name".to_owned(),
                other_field: "name".to_owned(),
            },
        );
        types.insert("Foo", foo_types);

        println!("{}", serde_json::to_string(&types).unwrap())
    }
}
