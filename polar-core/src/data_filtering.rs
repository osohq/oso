use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::counter::Counter;
use crate::error::{PolarError, PolarResult};
use crate::events::ResultEvent;
use crate::kb::Bindings;
use crate::terms::*;

use std::env;

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
pub struct Ref {
    field: Option<String>, // An optional field to map over the result objects with.
    result_id: String,     // Id of the FetchResult that should be an input.
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum ConstraintValue {
    Term(Term), // An actual value
    Ref(Ref)    // A reference to a different result.
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum ConstraintKind {
    Eq,        // The field is equal to a value.
    In,        // The field is equal to one of the values.
    Contains,  // The field is a collection that contains the value.
}

// @NOTE(steve): Constraint is sort of an overloaded word.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct Constraint {
    kind: ConstraintKind,
    field: String,
    value: ConstraintValue
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

impl FilterPlan {
    pub fn explain(&self) {
        eprintln!("==Filter Plan==");
        // For now each result set is union'd. This is a top level OR.
        // After I actually implement OR this will change.
        eprintln!("UNION");
        for (i, result_set) in self.result_sets.iter().enumerate() {
            eprintln!("  =Result Set: {}=", i);
            for id in &result_set.resolve_order {
                let fetch_request = result_set.requests.get(id).unwrap();
                eprintln!("    {}: Fetch {}", id, fetch_request.class_tag);
                for constraint in &fetch_request.constraints {
                    let op = match constraint.kind {
                        ConstraintKind::Eq => "=".to_owned(),
                        ConstraintKind::In => "in".to_owned(),
                        ConstraintKind::Contains => "contains".to_owned(),
                    };
                    let field = constraint.field.clone();
                    let value = match &constraint.value {
                        ConstraintValue::Term(t) => t.to_polar(),
                        ConstraintValue::Ref(r) => {
                            let mut s = "REF(".to_owned();
                            if let Some(field) = &r.field {
                                s.push_str(&format!("field {} of ", field));
                            }
                            s.push_str(&format!("result {}", r.result_id));
                            s
                        }
                    };
                    eprintln!("          {} {} {}", field, op, value);
                }
            }
        }
    }
}

pub type Types = HashMap<String, HashMap<String, Type>>;
pub type PartialResults = Vec<ResultEvent>;

#[derive(Debug)]
struct VarInfo {
    cycles: Vec<(Symbol, Symbol)>,  // x = y
    types: Vec<(Symbol, String)>,   // x matches XClass
    eq_values: Vec<(Symbol, Term)>, // x = 1;
    // in_values: Vec<(Symbol, Term)>, // x in [1,2,3]
    contained_values: Vec<(Term, Symbol)>, // 1 in x
    field_relationships: Vec<(Symbol, String, Symbol)>, // x.a = y
    in_relationships: Vec<(Symbol, Symbol)>,            // x in y
}

// @TODO(steve): Better way to handle these checks than just unwraps and asserts.

fn process_result(exp: &Operation) -> VarInfo {
    let mut var_info = VarInfo {
        cycles: vec![],
        types: vec![],
        eq_values: vec![],
        // in_values: vec![],
        contained_values: vec![],
        field_relationships: vec![],
        in_relationships: vec![]
    };
    process_exp(&mut var_info, exp);
    var_info
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
            let field = &exp.args[1];
            if let Ok(inner_exp) = var.value().as_expression() {
                assert_eq!(inner_exp.operator, Operator::Dot);
                var = process_exp(var_info, inner_exp).unwrap();
            }
            // TODO(steve): There's a potential name clash here which would be bad. Works for now.
            // but should probably generate this var better.
            let sym = var.value().as_symbol().unwrap();
            let field_str = field.value().as_string().unwrap();
            let new_var = Symbol::new(&format!("{}_dot_{}", sym.0, field_str));

            // Record the relationship between the vars.
            var_info
                .field_relationships
                .push((sym.clone(), field_str.to_string(), new_var.clone()));

            // Return the var so we can unify with it.
            return Some(Term::new_temporary(Value::Variable(new_var)));
        }
        Operator::Isa => {
            assert_eq!(exp.args.len(), 2);
            let lhs = &exp.args[0];
            let rhs = &exp.args[1];
            let var = lhs.value().as_symbol().unwrap();
            let pattern = rhs.value().as_pattern().unwrap();
            if let Pattern::Instance(InstanceLiteral { tag, fields }) = pattern {
                // @TODO(steve): Handle specializer fields.
                assert!(fields.fields.is_empty());
                var_info.types.push((var.clone(), tag.clone().0))
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
                    .eq_values
                    .push((var.clone(), Term::new_temporary(val.clone()))),
                // Unifying something else, I think would be an error in most cases???
                // 1 = 1 is irrelevant for data filtering, other stuff seems like an error.
                (a, b) => {
                    eprintln!("Bad unify: {} = {}", a.to_polar(), b.to_polar());
                    todo!()
                }
            };
        }
        Operator::In => {
            // So in is similar to unify, but is just talking about multiple values.
            // We *could* treat it as an `or`, but I think we probably don't want to do that.
            // variable in variable is a relationship between vars.
            // variable in list of values is a value relationship, and can probably directly translate to an in constraint.
            // what does value in variable mean? It's like a thing we'll have to check in the resolver?
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
                // l in r
                (Value::Variable(l), Value::Variable(r)) => {
                    var_info.in_relationships.push((l.clone(), r.clone()));
                }
                // var in [1, 2, 3]
                (Value::Variable(var), val) => {
                    // @Q(steve): Should I make sure this value is a list?
                    // @Q(steve): Does this ever actually come through the simplifier?
                    unimplemented!();
                    // var_info.in_values.push((var.clone(), Term::new_temporary(val.clone())));
                },
                // 123 in var
                (val, Value::Variable(var)) => {
                    var_info.contained_values.push((Term::new_temporary(val.clone()), var.clone()));
                }
                (a, b) => {
                    eprintln!("Bad in: {} in {}", a.to_polar(), b.to_polar());
                    todo!()
                }
            };

        }
        op => todo!("Unhandled Operation: {}", op.to_polar()),
    }
    None
}

#[derive(Debug)]
struct Vars {
    variables: HashMap<String, HashSet<Symbol>>,
    field_relationships: Vec<(String, String, String)>,
    in_relationships: Vec<(String, String)>,
    eq_values: HashMap<String, Term>,
    contained_values: HashMap<String, HashSet<Term>>,
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
    'relationships: for (parent, _, _) in &var_info.field_relationships {
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
    'relationships: for (_, _, child) in &var_info.field_relationships {
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
    let mut new_unifies: Vec<(String, String)> = vec![];
    for (i, ((parent_id1, child_id1), (_, field1, _))) in parent_ids
        .iter()
        .zip(child_ids.iter())
        .zip(var_info.field_relationships.iter())
        .enumerate()
    {
        for (j, ((parent_id2, child_id2), (_, field2, _))) in parent_ids
            .iter()
            .zip(child_ids.iter())
            .zip(var_info.field_relationships.iter())
            .enumerate()
        {
            if i != j && parent_id1 == parent_id2 && field1 == field2 && child_id1 != child_id2 {
                // Unify children
                new_unifies.push((child_id1.clone(), child_id2.clone()));
            }
        }
    }

    // @TODO(steve): There are absolutely bugs in here.
    // If we're turning 0 into 1 and then 0 into 2 it'll just blow up
    // not correctly turn 0 and 1 into 2. Needs some tests.
    for (x, y) in &new_unifies {
        eprint!("{} into {}", x, y);
        let mut xs = variables.remove(x).unwrap();
        let ys = variables.remove(y).unwrap();
        xs.extend(ys);
        variables.insert(x.clone(), xs);
    }

    // Substitute in relationship ids.
    // @Sorry(steve): This is a real mess too.
    let mut field_relationships = vec![];
    for (parent, field, child) in &var_info.field_relationships {
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
        field_relationships.push((parent_id, field.clone(), child_id));
    }

    // @TODO(steve): If we have duplicates in field_relationships, we can remove them. We already know.
    // Could use a set I suppose to handle that.

    // In relationships
    let mut in_relationships = vec![];
    for (lhs, rhs) in &var_info.in_relationships {
        let mut lhs_id = String::new();
        let mut rhs_id = String::new();
        for (id, set) in &mut variables {
            if set.contains(lhs) {
                lhs_id = id.clone();
            }
            if set.contains(rhs) {
                rhs_id = id.clone();
            }
        }
        if lhs_id == String::new() {
            let new_id = get_id();
            let mut new_set = HashSet::new();
            new_set.insert(lhs.clone());
            variables.insert(new_id.clone(), new_set);
        }
        if rhs_id == String::new() {
            let new_id = get_id();
            let mut new_set = HashSet::new();
            new_set.insert(rhs.clone());
            variables.insert(new_id.clone(), new_set);
        }
        in_relationships.push((lhs_id, rhs_id));
    }

    // I think a var can only have one value since we make sure there's a var for the dot lookup,
    // and if they had aliases they'd be collapsed by now, so it should be an error
    // if foo.name = "steve" and foo.name = "gabe".
    // TODO(steve): How are we going to handle "in"
    let mut eq_values = HashMap::new();
    'values: for (var, value) in var_info.eq_values {
        for (id, set) in &mut variables {
            if set.contains(&var) {
                // @TODO(steve): If we already have a value for it make sure they match don't just
                // overwrite it.
                eq_values.insert(id.clone(), value);
                continue 'values;
            }
        }
        // Create new variable if we didn't find one.
        let new_id = get_id();
        let mut new_set = HashSet::new();
        new_set.insert(var.clone());
        variables.insert(new_id.clone(), new_set);
        eq_values.insert(new_id, value);
    }

    let mut contained_values = HashMap::new();
    'contained_values: for (value, var) in var_info.contained_values {
        for (id, set) in &mut variables {
            if set.contains(&var) {
                contained_values.entry(id.clone()).or_insert(HashSet::new()).insert(value);
                continue 'contained_values;
            }
        }
        // Create new variable if we didn't find one.
        let new_id = get_id();
        let mut new_set = HashSet::new();
        new_set.insert(var.clone());
        variables.insert(new_id.clone(), new_set);
        let mut new_val_set = HashSet::new();
        new_val_set.insert(value);
        contained_values.insert(new_id, new_val_set);
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
        field_relationships,
        in_relationships,
        eq_values,
        contained_values,
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

    for (parent, field, child) in &vars.field_relationships {
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

                    request.constraints.push(
                        Constraint{
                            kind: ConstraintKind::In,
                            field: my_field.clone(),
                            value: ConstraintValue::Ref(Ref{
                                field: Some(other_field.clone()),
                                result_id: child.clone()
                            })
                        }
                    );
                    continue;
                }
            }
            // Non relationship or unknown type info.
            let mut contributed_constraints = false;
            if let Some(value) = vars.eq_values.get(child) {
                request.constraints.push(
                    Constraint{
                        kind: ConstraintKind::Eq,
                        field: field.clone(),
                        value: ConstraintValue::Term(value.clone())
                    }
                );
                contributed_constraints = true;
            }
            if let Some(values) = vars.contained_values.get(child) {
                for value in values {
                    request.constraints.push(
                        Constraint{
                            kind: ConstraintKind::Contains,
                            field: field.clone(),
                            value: ConstraintValue::Term(value.clone())
                        }
                    );
                }
                contributed_constraints = true;
            }
            assert!(contributed_constraints);
        }
    }

    // Constrain any vars that are `in` this var.
    // Add their constraints to this one.
    // @NOTE(steve): I think this is right, but I'm not totally sure.
    // This might assume that the current var is a relationship of a different type that
    // is of type "children".
    for (lhs, rhs) in &vars.in_relationships {
        if rhs == var_id {
            constrain_var(result_set, types, vars, lhs, var_type);
            let in_result_set = result_set.requests.remove(lhs).unwrap();
            assert_eq!(result_set.resolve_order.pop(), Some(lhs.to_string()));
            request.constraints.extend(in_result_set.constraints);
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
    let mut filter_plan = FilterPlan {
        result_sets: vec![],
    };
    // @NOTE(steve): Just reading an env var here sucks (see all the stuff we had to do
    // to get POLAR_LOG to work in all libs, wasm etc...) but that's what I'm doing today.
    // At some point surface this info better.
    let explain = match std::env::var("POLAR_EXPLAIN") {
        Ok(_) => true,
        Err(_) => false
    };

    if explain {
        eprintln!("===Data Filtering Query===");
        eprintln!("==Bindings==")
    }

    // @NOTE(steve): For now we build a ResultSet for each result. Then we put them into a filterplan
    // which effectively means the results should all be UNION'd together.
    // I suspect this structure will change a little bit once we introduce OR.
    for (i, result) in partial_results.iter().enumerate() {
        let term = result.bindings.get(&Symbol::new(variable)).unwrap();
        let exp = term.value().as_expression()?;
        assert_eq!(exp.operator, Operator::And);

        if explain {
            eprintln!("  {}: {}", i, term.to_polar());
        }

        let var_info = process_result(exp);
        let vars = collapse_vars(var_info);

        if explain {
            eprintln!("    variables");
            for (id, set) in &vars.variables {
                let values = set.clone().into_iter().map(|sym|{sym.0.to_owned()}).collect::<Vec<String>>().join(", ");
                eprintln!("      {}:  vars: {{{}}}", id, values);
                let type_tag = if let Some(tag) = vars.types.get(id) {
                    tag.clone()
                } else if let Some(val) = vars.eq_values.get(id) {
                    match val.value() {
                        Value::Boolean(_) => {
                            "Bool".to_owned()
                        },
                        Value::String(_) => {
                            "String".to_owned()
                        },
                        Value::Number(_) => {
                            "Number".to_owned()
                        },
                        Value::List(_) => {
                            "List".to_owned()
                        }
                        Value::Dictionary(_) => {
                            "Dictionary".to_owned()
                        },
                        _ => todo!()
                    }
                } else {
                    "unknown".to_owned()
                };
                eprintln!("          type: {}", type_tag);
                if let Some(val) = vars.eq_values.get(id) {
                    eprintln!("          value: {}", val.to_polar());
                }
                if let Some(values) = vars.contained_values.get(id) {
                    for val in values {
                        eprintln!("          value contains: {}", val.to_polar());
                    }
                }
            }
        }
        eprintln!("    field relationships");
        for (x,field,y) in &vars.field_relationships {
            eprintln!("      {}.{} = {}", x, field, y);
        }
        eprintln!("    in relationships");
        for (x,y) in &vars.in_relationships {
            eprintln!("      {} in {}", x, y);
        }

        let result_set = constrain_vars(&types, &vars, class_tag);
        filter_plan.result_sets.push(result_set);
    }

    if explain {
        filter_plan.explain()
    }

    Ok(filter_plan)
}


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

        println!("{}", serde_json::to_string(&types).unwrap());

        let r = Ref{field: None, result_id: "123".to_string() };
        println!("{}", serde_json::to_string(&r).unwrap());
    }
}
