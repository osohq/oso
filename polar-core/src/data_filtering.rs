use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::error::PolarResult;
use crate::events::ResultEvent;

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
pub struct Ref {
    field: Option<String>, // An optional field to map over the result objects with.
    result_id: String,     // Id of the FetchResult that should be an input.
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum ConstraintValue {
    Term(Term),    // An actual value
    Ref(Ref),      // A reference to a different result.
    Field(String), // Another field on the same result
}

// @TODO(steve): These are all constraints on a field. If we need to add constraints
// on the value itself. eg `value in [Foo{id: "blah}]` then we should probably call
// these FieldEq, FieldIn, FieldContains
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum ConstraintKind {
    Eq,       // The field is equal to a value.
    In,       // The field is equal to one of the values.
    Contains, // The field is a collection that contains the value.
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct Constraint {
    kind: ConstraintKind,
    field: String,
    value: ConstraintValue,
}

// The list of constraints passed to a fetching function for a particular type.
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

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct FilterPlan {
    result_sets: Vec<ResultSet>,
}

impl FilterPlan {
    pub fn explain(&self) {
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
                        ConstraintValue::Field(f) => format!("FIELD({})", f),
                        ConstraintValue::Ref(r) => {
                            let mut s = "REF(".to_owned();
                            if let Some(field) = &r.field {
                                s.push_str(&format!("field {} of ", field));
                            }
                            s.push_str(&format!("result {})", r.result_id));
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
    cycles: Vec<(Symbol, Symbol)>,                      // x = y
    types: Vec<(Symbol, String)>,                       // x matches XClass
    eq_values: Vec<(Symbol, Term)>,                     // x = 1;
    contained_values: Vec<(Term, Symbol)>,              // 1 in x
    field_relationships: Vec<(Symbol, String, Symbol)>, // x.a = y
    in_relationships: Vec<(Symbol, Symbol)>,            // x in y
}

// @TODO(steve): Better way to handle these checks than just unwraps and asserts.

fn process_result(exp: &Operation) -> VarInfo {
    let mut var_info = VarInfo {
        cycles: vec![],
        types: vec![],
        eq_values: vec![],
        contained_values: vec![],
        field_relationships: vec![],
        in_relationships: vec![],
    };
    process_exp(&mut var_info, exp);
    var_info
}

fn dot_var(var_info: &mut VarInfo, var: Term, field: &Term) -> Symbol {
    let mut var = var;
    while let Ok(Operation {
        operator: Operator::Dot,
        args,
    }) = var.value().as_expression()
    {
        var = Term::new_temporary(Value::Variable(dot_var(
            var_info,
            args[0].clone(),
            &args[1],
        )))
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
                if inner_exp.operator != Operator::Dot {
                    unimplemented!("Operations other than dot nested within a dot are not yet supported for data filtering.")
                }
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
            if let Value::Pattern(Pattern::Instance(InstanceLiteral { tag, fields })) = rhs.value()
            {
                if !fields.fields.is_empty() {
                    unimplemented!("Specializer fields are not yet supported for data filtering.")
                }
                assert!(fields.fields.is_empty());
                let var = match lhs.value() {
                    Value::Variable(var) | Value::RestVariable(var) => var.clone(),
                    Value::Expression(op) if op.operator == Operator::Dot => {
                        dot_var(var_info, op.args[0].clone(), &op.args[1])
                    }
                    _ => todo!(),
                };
                var_info.types.push((var, tag.clone().0))
            } else {
                unimplemented!("Non pattern specializers are not yet supported for data filtering.")
            }
        }
        Operator::Unify | Operator::Eq | Operator::Assign => {
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
                // Unifying something else.
                // 1 = 1 is irrelevant for data filtering, other stuff seems like an error.
                // @NOTE(steve): Going with the same not yet supported message but if this is
                // coming through it's probably a bug in the simplifier.
                (_a, _b) => {
                    unimplemented!(
                        "Unification of values is not yet supported for data filtering."
                    );
                }
            };
        }
        Operator::In => {
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
                (Value::Variable(_var), _val) => {
                    // @Q(steve): Does this ever actually come through the simplifier?
                    // @Note(steve): MikeD wishes this came through as an in instead of or-expanded.
                    // That way we could turn it into an `in` in sql.
                    unimplemented!("var in list of values constraints are not yet supported for data filtering.");
                    // var_info.in_values.push((var.clone(), Term::new_temporary(val.clone())));
                }
                // 123 in var
                (val, Value::Variable(var)) => {
                    var_info
                        .contained_values
                        .push((Term::new_temporary(val.clone()), var.clone()));
                }
                (_a, _b) => {
                    // @NOTE: This is probably just a bug if we hit it. Shouldn't get any other `in` cases.
                    unimplemented!(
                        "Unknown `in` constraint that is not yet supported for data filtering."
                    );
                }
            };
        }
        Operator::Debug => unimplemented!("debug() is not supported for data filtering."),
        Operator::Print => (),
        Operator::Cut => unimplemented!("`cut` is not supported for data filtering."),
        Operator::New => panic!("`new` operation in expression"),
        Operator::Not => panic!("`not` operation in expression"),
        Operator::Mul => unimplemented!("multiplication is not supported for data filtering."),
        Operator::Div => unimplemented!("division is not supported for data filtering."),
        Operator::Mod => unimplemented!("`mod` is not supported for data filtering."),
        Operator::Rem => unimplemented!("`rem` is not supported for data filtering."),
        Operator::Add => unimplemented!("addition is not supported for data filtering."),
        Operator::Sub => unimplemented!("subtraction is not supported for data filtering."),
        Operator::Geq => unimplemented!("`>=` is not supported for data filtering."),
        Operator::Leq => unimplemented!("`<=` is not supported for data filtering."),
        Operator::Neq => unimplemented!("`!=` is not supported for data filtering."),
        Operator::Gt => unimplemented!("`>` is not supported for data filtering."),
        Operator::Lt => unimplemented!("`<` is not supported for data filtering."),
        // @TODO(steve): Expand or expressions to multiple bindings in the simplifier.
        Operator::Or => unimplemented!("`or` is not supported for data filtering."),
        Operator::ForAll => unimplemented!("`forall` is not supported for data filtering."),
    }
    None
}

#[derive(Debug)]
struct Vars {
    variables: HashMap<String, HashSet<Symbol>>,
    field_relationships: HashSet<(String, String, String)>,
    in_relationships: HashSet<(String, String)>,
    eq_values: HashMap<String, Term>,
    contained_values: HashMap<String, HashSet<Term>>,
    types: HashMap<String, String>,
    this_id: String,
}

/// Collapses the var info that we obtained from walking the expressions.
/// Track equivalence classes of variables and assign each one an id.
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
    'parent_relationships: for (parent, _, _) in &var_info.field_relationships {
        for (id, set) in &mut variables {
            if set.contains(parent) {
                parent_ids.push(id.clone());
                continue 'parent_relationships;
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
    'child_relationships: for (_, _, child) in &var_info.field_relationships {
        for (id, set) in &mut variables {
            if set.contains(child) {
                child_ids.push(id.clone());
                continue 'child_relationships;
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
    // @Sorry(steve): Wow, what a loop. Maybe just loop over indexes instead.
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
        if x != y {
            let mut xs = variables.remove(x).unwrap();
            let ys = variables.remove(y).unwrap();
            xs.extend(ys);
            variables.insert(x.clone(), xs);
        }
    }

    // Substitute in relationship ids.
    // @Sorry(steve): This is a real mess too.
    let mut field_relationships = HashSet::new();
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
        field_relationships.insert((parent_id, field.clone(), child_id));
    }

    // In relationships
    let mut in_relationships = HashSet::new();
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
        in_relationships.insert((lhs_id, rhs_id));
    }

    // I think a var can only have one value since we make sure there's a var for the dot lookup,
    // and if they had aliases they'd be collapsed by now, so it should be an error
    // if foo.name = "steve" and foo.name = "gabe".
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
                contained_values
                    .entry(id.clone())
                    .or_insert_with(HashSet::new)
                    .insert(value);
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
    constrain_var(&mut result_set, types, vars, &vars.this_id, this_type);
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
    let type_def = types
        .iter()
        .find(|r| r.0 == var_type)
        .map(|r| r.1.clone())
        .unwrap_or_else(HashMap::new);

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
            let typ = match type_def.get(field) {
                None => panic!("unknown field {}", field),
                Some(t) => t,
            };

            if let Type::Relationship {
                other_class_tag,
                my_field,
                other_field,
                ..
            } = typ
            {
                constrain_var(result_set, types, vars, child, other_class_tag);

                // If the constrained child var doesn't have any constraints on it, we don't need to
                // constrain this var. Otherwise we're just saying field foo in all Foos which
                // would fetch all Foos and not be good.
                if let Some(child_result) = result_set.requests.remove(child) {
                    if !child_result.constraints.is_empty() {
                        result_set.requests.insert(child.to_owned(), child_result);
                        request.constraints.push(Constraint {
                            kind: ConstraintKind::In,
                            field: my_field.clone(),
                            value: ConstraintValue::Ref(Ref {
                                field: Some(other_field.clone()),
                                result_id: child.clone(),
                            }),
                        });
                    } else {
                        // Remove the id from the resolve_order too.
                        result_set.resolve_order.pop();
                    }
                }

                continue;
            }
            // Non relationship or unknown type info.
            let mut contributed_constraints = false;
            if let Some(value) = vars.eq_values.get(child) {
                request.constraints.push(Constraint {
                    kind: ConstraintKind::Eq,
                    field: field.clone(),
                    value: ConstraintValue::Term(value.clone()),
                });
                contributed_constraints = true;
            }
            if let Some(values) = vars.contained_values.get(child) {
                for value in values {
                    request.constraints.push(Constraint {
                        kind: ConstraintKind::Contains,
                        field: field.clone(),
                        value: ConstraintValue::Term(value.clone()),
                    });
                }
                contributed_constraints = true;
            }
            for eqf in vars
                .field_relationships
                .iter()
                .filter(|r| r.0 == *parent && r.1 != *field && r.2 == *child)
            {
                request.constraints.push(Constraint {
                    kind: ConstraintKind::Eq,
                    field: field.clone(),
                    value: ConstraintValue::Field(eqf.1.clone()),
                });
                contributed_constraints = true;
            }
            assert!(contributed_constraints);
        }
    }

    // Constrain any vars that are `in` this var.
    // Add their constraints to this one.
    // @NOTE(steve): I think this is right, but I'm not totally sure.
    // This might assume that the current var is a relationship of kind "children".
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

pub fn opt_pass(filter_plan: &mut FilterPlan, explain: bool) -> bool {
    let mut optimized = false;

    // Remove duplicate result set in a union.
    let mut drop_plan = None;
    'plans: for (i, result_set_a) in filter_plan.result_sets.iter().enumerate() {
        for (j, result_set_b) in filter_plan.result_sets.iter().enumerate() {
            if i != j && result_set_a == result_set_b {
                drop_plan = Some(j);
                break 'plans;
            }
        }
    }
    if let Some(plan_id) = drop_plan {
        if explain {
            eprintln!("* Removed duplicate result set.")
        }
        filter_plan.result_sets.remove(plan_id);
        optimized = true;
    }

    // Possible future optimization ideas.
    // * If two result sets are almost the same except for a single fetch
    //   that only has a single field check and the field is different, we
    //   can merge the two result sets and turn the field check into an `in`.
    //   This is basically "un-expanding" either an `in` or and `or` from the policy.
    //   This could be hard to find.
    optimized
}

pub fn optimize(mut filter_plan: FilterPlan, explain: bool) -> FilterPlan {
    if explain {
        eprintln!("\nOptimizing...")
    }
    while opt_pass(&mut filter_plan, explain) {}
    if explain {
        eprintln!("Done\n")
    }
    filter_plan
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
    let explain = std::env::var("POLAR_EXPLAIN").is_ok();

    if explain {
        eprintln!("\n===Data Filtering Query===");
        eprintln!("\n==Bindings==")
    }

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
                let values = set
                    .clone()
                    .into_iter()
                    .map(|sym| sym.0)
                    .collect::<Vec<String>>()
                    .join(", ");
                eprintln!("      {}:  vars: {{{}}}", id, values);
                let type_tag = if let Some(tag) = vars.types.get(id) {
                    tag.clone()
                } else if let Some(val) = vars.eq_values.get(id) {
                    match val.value() {
                        Value::Boolean(_) => "Bool".to_owned(),
                        Value::String(_) => "String".to_owned(),
                        Value::Number(_) => "Number".to_owned(),
                        Value::List(_) => "List".to_owned(),
                        Value::Dictionary(_) => "Dictionary".to_owned(),
                        _ => todo!(),
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
            eprintln!("    field relationships");
            for (x, field, y) in &vars.field_relationships {
                eprintln!("      {}.{} = {}", x, field, y);
            }
            eprintln!("    in relationships");
            for (x, y) in &vars.in_relationships {
                eprintln!("      {} in {}", x, y);
            }
        }

        let result_set = constrain_vars(&types, &vars, class_tag);
        filter_plan.result_sets.push(result_set);
    }

    if explain {
        eprintln!("== Raw Filter Plan ==");
        filter_plan.explain()
    }

    let opt_filter_plan = optimize(filter_plan, explain);
    if explain {
        eprintln!("== Optimized Filter Plan ==");
        opt_filter_plan.explain()
    }

    Ok(opt_filter_plan)
}
