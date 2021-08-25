use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::error::PolarResult;
use crate::events::ResultEvent;

use crate::counter::*;
use crate::terms::*;
use std::hash::Hash;

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
    result_id: Id,         // Id of the FetchResult that should be an input.
}

type Id = u64;

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
    Neq,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct Constraint {
    kind: ConstraintKind,
    field: Option<String>,
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
    requests: HashMap<Id, FetchRequest>,
    resolve_order: Vec<Id>,
    result_id: Id,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default)]
pub struct FilterPlan {
    result_sets: Vec<ResultSet>,
}

pub type Types = HashMap<String, HashMap<String, Type>>;
pub type PartialResults = Vec<ResultEvent>;

#[derive(Debug, Default)]
struct VarInfo {
    cycles: Vec<(Symbol, Symbol)>,                      // x = y
    uncycles: Vec<(Symbol, Symbol)>,                    // x != y
    types: Vec<(Symbol, String)>,                       // x matches XClass
    eq_values: Vec<(Symbol, Term)>,                     // x = 1
    neq_values: Vec<(Symbol, Term)>,                    // x != 1
    contained_values: Vec<(Term, Symbol)>,              // 1 in x
    field_relationships: Vec<(Symbol, String, Symbol)>, // x.a = y
    in_relationships: Vec<(Symbol, Symbol)>,            // x in y
    counter: Counter,
}

#[derive(Debug)]
struct Vars {
    variables: HashMap<Id, HashSet<Symbol>>,
    field_relationships: HashSet<(Id, String, Id)>,
    uncycles: HashSet<(Id, Id)>,
    in_relationships: HashSet<(Id, Id)>,
    eq_values: HashMap<Id, Term>,
    neq_values: HashSet<(Id, Term)>,
    contained_values: HashMap<Id, HashSet<Term>>,
    types: HashMap<Id, String>,
    this_id: Id,
}

pub fn build_filter_plan(
    types: Types,
    partial_results: PartialResults,
    variable: &str,
    class_tag: &str,
) -> PolarResult<FilterPlan> {
    Ok(FilterPlan::new(types, partial_results, variable, class_tag))
}

impl From<&Operation> for VarInfo {
    fn from(op: &Operation) -> Self {
        let mut info = Self::default();
        info.process_exp(op);
        info
    }
}

impl From<&Operation> for Vars {
    fn from(op: &Operation) -> Self {
        VarInfo::from(op).into()
    }
}

impl From<VarInfo> for Vars {
    /// Collapses the var info that we obtained from walking the expressions.
    /// Track equivalence classes of variables and assign each one an id.
    fn from(info: VarInfo) -> Self {
        /// try to find an existing id for this variable.
        fn seek_var_id(vars: &HashMap<Id, HashSet<Symbol>>, var: &Symbol) -> Option<Id> {
            vars.iter()
                .find_map(|(id, set)| set.contains(var).then(|| *id))
        }

        /// get the id for this variable, or create one if the variable is new.
        fn get_var_id(
            vars: &mut HashMap<Id, HashSet<Symbol>>,
            var: Symbol,
            counter: &Counter,
        ) -> Id {
            seek_var_id(vars, &var).unwrap_or_else(|| {
                let new_id = counter.next();
                let mut new_set = HashSet::new();
                new_set.insert(var);
                vars.insert(new_id, new_set);
                new_id
            })
        }

        let counter = info.counter;

        // group the variables into equivalence classes.
        let mut variables = partition_equivs(info.cycles)
            // Give each cycle an id
            .into_iter()
            .map(|c| (counter.next(), c))
            .collect::<HashMap<_, _>>();

        let fields = info.field_relationships;
        let mut assign_id = |item| get_var_id(&mut variables, item, &counter);

        let uncycles = info
            .uncycles
            .into_iter()
            .map(|(a, b)| canonical_pair(assign_id(a), assign_id(b)))
            .collect();

        // now convert the remaining VarInfo fields into equivalent Vars fields.

        let in_relationships = info
            .in_relationships
            .into_iter()
            .map(|(lhs, rhs)| (assign_id(lhs), assign_id(rhs)))
            .collect::<HashSet<_>>();

        // I think a var can only have one value since we make sure there's a var for the dot lookup,
        // and if they had aliases they'd be collapsed by now, so it should be an error
        // if foo.name = "steve" and foo.name = "gabe".
        let eq_values = info
            .eq_values
            .into_iter()
            .map(|(var, val)| (assign_id(var), val))
            .collect::<HashMap<_, _>>();

        let neq_values = info
            .neq_values
            .into_iter()
            .map(|(var, val)| (assign_id(var), val))
            .collect::<HashSet<_>>();

        let types = info
            .types
            .into_iter()
            .map(|(var, typ)| (assign_id(var), typ))
            .collect::<HashMap<_, _>>();

        let contained_values =
            info.contained_values
                .into_iter()
                .fold(HashMap::new(), |mut map, (val, var)| {
                    map.entry(assign_id(var))
                        .or_insert_with(HashSet::new)
                        .insert(val);
                    map
                });

        let field_relationships = fields
            .into_iter()
            .map(|(p, f, c)| (assign_id(p), f, assign_id(c)))
            .collect::<HashSet<_>>();

        let this_id = seek_var_id(&variables, &sym!("_this")).expect("nothing to filter for!");

        Vars {
            variables,
            uncycles,
            field_relationships,
            in_relationships,
            eq_values,
            neq_values,
            contained_values,
            types,
            this_id,
        }
    }
}

impl VarInfo {
    fn dot_var(&mut self, var: &Term, field: &Term) -> Symbol {
        // handle nested dot ops.
        let var = self.eval(var);

        let sym = var.as_symbol().unwrap();
        let field_str = field.value().as_string().unwrap();

        if let Some(var) = self
            .field_relationships
            .iter()
            .find_map(|(p, f, c)| (p == sym && f == field_str).then(|| c))
        {
            return var.clone();
        }

        let new_var = sym!(&format!(
            "_{}_dot_{}_{}",
            sym.0,
            field_str,
            self.counter.next()
        ));

        // Record the relationship between the vars.
        self.field_relationships
            .push((sym.clone(), field_str.to_string(), new_var.clone()));

        new_var
    }

    fn eval(&mut self, term: &Term) -> Value {
        match term.value() {
            Value::Expression(Operation {
                operator: Operator::Dot,
                args,
            }) if args.len() == 2 => self.dot_var(&args[0], &args[1]).into(),
            v => v.clone(),
        }
    }

    /// Process an expression in the context of this VarInfo. Mostly about the side effects.
    fn process_exp(&mut self, exp: &Operation) {
        match exp.operator {
            Operator::And => {
                for arg in &exp.args {
                    let inner_exp = arg.value().as_expression().unwrap();
                    self.process_exp(inner_exp);
                }
            }
            Operator::Dot => {
                // Dot operations return a var that can be unified with.
                // We create a new var to represent the result of the operation.
                self.dot_var(&exp.args[0], &exp.args[1]);
            }
            Operator::Isa => {
                assert_eq!(exp.args.len(), 2);
                let (lhs, rhs) = (&exp.args[0], &exp.args[1]);
                if let Ok(Pattern::Instance(InstanceLiteral { tag, fields })) =
                    rhs.value().as_pattern()
                {
                    if !fields.fields.is_empty() {
                        unimplemented!(
                            "Specializer fields are not yet supported for data filtering."
                        )
                    }
                    let var = match self.eval(lhs) {
                        Value::Variable(var) | Value::RestVariable(var) => var,
                        _ => todo!(),
                    };
                    self.types.push((var, tag.0.clone()))
                } else {
                    unimplemented!(
                        "Non pattern specializers are not yet supported for data filtering."
                    )
                }
            }
            Operator::Unify | Operator::Eq | Operator::Assign => {
                assert_eq!(exp.args.len(), 2);

                match (self.eval(&exp.args[0]), self.eval(&exp.args[1])) {
                    // Unifying two variables
                    (Value::Variable(l), Value::Variable(r)) => self.cycles.push((l, r)),
                    // Unifying a variable with a value
                    (Value::Variable(var), val) | (val, Value::Variable(var)) => {
                        self.eq_values.push((var, Term::from(val)))
                    }
                    // Unifying something else.
                    // 1 = 1 is irrelevant for data filtering, other stuff seems like an error.
                    // @NOTE(steve): Going with the same not yet supported message but if this is
                    // coming through it's probably a bug in the simplifier.
                    _ => unimplemented!(
                        "Unification of values is not yet supported for data filtering."
                    ),
                };
            }
            Operator::Neq => {
                assert_eq!(exp.args.len(), 2);

                match (self.eval(&exp.args[0]), self.eval(&exp.args[1])) {
                    // Unifying two variables
                    (Value::Variable(l), Value::Variable(r)) => self.uncycles.push((l, r)),
                    // Unifying a variable with a value
                    (Value::Variable(var), val) | (val, Value::Variable(var)) => {
                        self.neq_values.push((var, Term::from(val)))
                    }
                    // Unifying something else.
                    // 1 = 1 is irrelevant for data filtering, other stuff seems like an error.
                    // @NOTE(steve): Going with the same not yet supported message but if this is
                    // coming through it's probably a bug in the simplifier.
                    _ => unimplemented!(
                        "Unification of values is not yet supported for data filtering."
                    ),
                };
            }
            Operator::In => {
                assert_eq!(exp.args.len(), 2);

                match (self.eval(&exp.args[0]), self.eval(&exp.args[1])) {
                    // l in r
                    (Value::Variable(l), Value::Variable(r)) =>
                        self.in_relationships.push((l, r)),
                    // var in [1, 2, 3]
                    (Value::Variable(_var), _val) =>
                        // @Q(steve): Does this ever actually come through the simplifier?
                        // @Note(steve): MikeD wishes this came through as an in instead of or-expanded.
                        // That way we could turn it into an `in` in sql.
                        unimplemented!("var in list of values constraints are not yet supported for data filtering."),
                        // self.in_values.push((var.clone(), Term::from(val.clone())));
                    // 123 in var
                    (val, Value::Variable(var)) =>
                        self.contained_values
                            .push((Term::from(val), var)),
                    _ =>
                        // @NOTE: This is probably just a bug if we hit it. Shouldn't get any other `in` cases.
                        unimplemented!(
                            "Unknown `in` constraint that is not yet supported for data filtering."
                        ),
                };
            }

            Operator::Print => (),
            x => unimplemented!(
                "`{}` is not yet supported for data filtering.",
                x.to_polar()
            ),
        }
    }
}

impl FilterPlan {
    fn new(
        types: Types,
        partial_results: PartialResults,
        var: &str,
        class_tag: &str,
    ) -> FilterPlan {
        // @NOTE(steve): Just reading an env var here sucks (see all the stuff we had to do
        // to get POLAR_LOG to work in all libs, wasm etc...) but that's what I'm doing today.
        // At some point surface this info better.
        let explain = std::env::var("POLAR_EXPLAIN").is_ok();

        if explain {
            eprintln!("\n===Data Filtering Query===");
            eprintln!("\n==Bindings==")
        }

        let result_sets = partial_results
            .into_iter()
            .enumerate()
            // if the result doesn't include a binding for this variable,
            // or if the binding isn't an expression, then just ignore it.
            .filter_map(|(i, result)| {
                result.bindings.get(&Symbol::new(var)).and_then(|term| {
                    term.value().as_expression().ok().map(|exp| {
                        assert_eq!(exp.operator, Operator::And);
                        let vars = Vars::from(exp);
                        if explain {
                            eprintln!("  {}: {}", i, term.to_polar());
                            vars.explain()
                        }

                        ResultSet::new(&types, &vars, class_tag)
                    })
                })
            })
            .collect();

        FilterPlan { result_sets }.optimize(explain)
    }

    fn opt_pass(&mut self, explain: bool) -> bool {
        let mut optimized = false;

        // Remove duplicate result set in a union.
        let drop_plan = self.result_sets.iter().enumerate().find_map(|(i, rs1)| {
            self.result_sets
                .iter()
                .enumerate()
                .find_map(|(j, rs2)| (i != j && rs1 == rs2).then(|| j))
        });

        if let Some(plan_id) = drop_plan {
            if explain {
                eprintln!("* Removed duplicate result set.")
            }
            self.result_sets.remove(plan_id);
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

    fn optimize(mut self, explain: bool) -> FilterPlan {
        if explain {
            eprintln!("== Raw Filter Plan ==");
            self.explain();
            eprintln!("\nOptimizing...")
        }

        while self.opt_pass(explain) {}

        if explain {
            eprintln!("Done\n");
            eprintln!("== Optimized Filter Plan ==");
            self.explain()
        }

        self
    }

    fn explain(&self) {
        eprintln!("UNION");
        for (i, result_set) in self.result_sets.iter().enumerate() {
            eprintln!("  =Result Set: {}=", i);
            for id in &result_set.resolve_order {
                let fetch_request = result_set.requests.get(id).unwrap();
                eprintln!("    {}: Fetch {}", id, fetch_request.class_tag);
                for constraint in &fetch_request.constraints {
                    let op = match constraint.kind {
                        ConstraintKind::Eq => "=",
                        ConstraintKind::In => "in",
                        ConstraintKind::Neq => "!=",
                        ConstraintKind::Contains => "contains",
                    };
                    let field = &constraint.field;
                    let value = match &constraint.value {
                        ConstraintValue::Term(t) => t.to_polar(),
                        ConstraintValue::Field(f) => format!("FIELD({})", f),
                        ConstraintValue::Ref(r) => {
                            let inside = match &r.field {
                                Some(f) => format!("{}.{}", r.result_id, f),
                                _ => format!("{}", r.result_id),
                            };
                            format!("REF({})", inside)
                        }
                    };
                    eprintln!("          {:?} {} {}", field, op, value);
                }
            }
        }
    }
}

impl ResultSet {
    fn new(types: &Types, vars: &Vars, this_type: &str) -> ResultSet {
        let mut result_set = ResultSet {
            requests: HashMap::new(),
            resolve_order: vec![],
            result_id: vars.this_id,
        };
        let mut seen = HashSet::new();
        result_set.constrain(types, vars, vars.this_id, this_type, &mut seen);
        result_set
    }

    fn constrain(
        &mut self,
        types: &Types,
        vars: &Vars,
        var_id: Id,
        var_type: &str,
        seen: &mut HashSet<Id>,
    ) {
        if seen.contains(&var_id) {
            return;
        }
        seen.insert(var_id);
        // @TODO(steve): Probably should check the type against the var types. I think???
        let type_def = types
            .iter()
            .find_map(|(l, r)| (l == var_type).then(|| r.clone()))
            .unwrap_or_else(HashMap::new);

        let mut request = self
            .requests
            .remove(&var_id)
            .unwrap_or_else(|| FetchRequest {
                class_tag: var_type.to_string(),
                constraints: vec![],
            });

        for (parent, field, child) in &vars.field_relationships {
            if *parent == var_id {
                if let Some(Type::Relationship {
                    other_class_tag,
                    my_field,
                    other_field,
                    ..
                }) = type_def.get(field)
                {
                    self.constrain(types, vars, *child, other_class_tag, seen);

                    // If the constrained child var doesn't have any constraints on it, we don't need to
                    // constrain this var. Otherwise we're just saying field foo in all Foos which
                    // would fetch all Foos and not be good.
                    if let Some(child_result) = self.requests.remove(child) {
                        if child_result.constraints.is_empty() {
                            // Remove the id from the resolve_order too.
                            self.resolve_order.pop();
                        } else {
                            self.requests.insert(child.to_owned(), child_result);
                            request.constraints.push(Constraint {
                                kind: ConstraintKind::In,
                                field: Some(my_field.clone()),
                                value: ConstraintValue::Ref(Ref {
                                    field: Some(other_field.clone()),
                                    result_id: *child,
                                }),
                            });
                        }
                    }

                    continue;
                }
                // Non relationship or unknown type info.
                let mut contributed_constraints = false;
                if let Some(value) = vars.eq_values.get(child) {
                    request.constraints.push(Constraint {
                        kind: ConstraintKind::Eq,
                        field: Some(field.clone()),
                        value: ConstraintValue::Term(value.clone()),
                    });
                    contributed_constraints = true;
                }

                vars.neq_values
                    .iter()
                    .filter_map(|(k, v)| (k == child).then(|| v))
                    .for_each(|v| {
                        request.constraints.push(Constraint {
                            kind: ConstraintKind::Neq,
                            field: Some(field.clone()),
                            value: ConstraintValue::Term(v.clone()),
                        });
                        contributed_constraints = true;
                    });
                if let Some(values) = vars.contained_values.get(child) {
                    for value in values {
                        request.constraints.push(Constraint {
                            kind: ConstraintKind::Contains,
                            field: Some(field.clone()),
                            value: ConstraintValue::Term(value.clone()),
                        });
                    }
                    contributed_constraints = true;
                }
                for (p, f, c) in vars.field_relationships.iter() {
                    if p == parent && f != field {
                        if c == child {
                            request.constraints.push(Constraint {
                                kind: ConstraintKind::Eq,
                                field: Some(field.clone()),
                                value: ConstraintValue::Field(f.clone()),
                            });
                            contributed_constraints = true;
                            continue;
                        }
                        let pair = canonical_pair(*c, *child);
                        if vars.uncycles.iter().any(|p| *p == pair) {
                            request.constraints.push(Constraint {
                                kind: ConstraintKind::Neq,
                                field: Some(field.clone()),
                                value: ConstraintValue::Field(f.clone()),
                            });
                            contributed_constraints = true;
                            continue;
                        }
                    }
                }
                assert!(contributed_constraints);
            }
        }

        // Constrain any vars that are `in` this var.
        // Add their constraints to this one.
        // @NOTE(steve): I think this is right, but I'm not totally sure.
        // This might assume that the current var is a relationship of kind "children".
        for l in vars
            .in_relationships
            .iter()
            .filter_map(|(l, r)| (*r == var_id).then(|| l))
        {
            self.constrain(types, vars, *l, var_type, seen);
            if let Some(in_result_set) = self.requests.remove(l) {
                assert_eq!(self.resolve_order.pop().unwrap(), *l);
                request.constraints.extend(in_result_set.constraints);
            }
        }

        vars.neq_values
            .iter()
            .filter_map(|(k, v)| (k == &var_id).then(|| v))
            .for_each(|v| {
                request.constraints.push(Constraint {
                    kind: ConstraintKind::Neq,
                    field: None,
                    value: ConstraintValue::Term(v.clone()),
                });
            });

        if let Some(vs) = vars.contained_values.get(&var_id) {
            for l in vs {
                request.constraints.push(Constraint {
                    kind: ConstraintKind::Eq,
                    field: None,
                    value: ConstraintValue::Term(l.clone()),
                });
            }
        }

        if let Some(l) = vars.eq_values.get(&var_id) {
            request.constraints.push(Constraint {
                kind: ConstraintKind::Eq,
                field: None,
                value: ConstraintValue::Term(l.clone()),
            });
        }

        self.requests.insert(var_id, request);
        self.resolve_order.push(var_id);
    }
}

impl Vars {
    fn explain(&self) {
        eprintln!("    variables");
        for (id, set) in &self.variables {
            let values = set
                .iter()
                .map(|sym| sym.0.clone())
                .collect::<Vec<String>>()
                .join(", ");
            eprintln!("      {}:  vars: {{{}}}", id, values);
            let type_tag = if let Some(tag) = self.types.get(id) {
                tag
            } else if let Some(val) = self.eq_values.get(id) {
                match val.value() {
                    Value::Boolean(_) => "Bool",
                    Value::String(_) => "String",
                    Value::Number(_) => "Number",
                    Value::List(_) => "List",
                    Value::Dictionary(_) => "Dictionary",
                    _ => todo!(),
                }
            } else {
                "unknown"
            };
            eprintln!("          type: {}", type_tag);
            if let Some(val) = self.eq_values.get(id) {
                eprintln!("          value: {}", val.to_polar());
            }
            if let Some(values) = self.contained_values.get(id) {
                for val in values {
                    eprintln!("          value contains: {}", val.to_polar());
                }
            }
        }
        eprintln!("    field relationships");
        for (x, field, y) in &self.field_relationships {
            eprintln!("      {}.{} = {}", x, field, y);
        }
        eprintln!("    in relationships");
        for (x, y) in &self.in_relationships {
            eprintln!("      {} in {}", x, y);
        }
    }
}

fn canonical_pair<A>(a: A, b: A) -> (A, A)
where
    A: Ord,
{
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}

/// generate equivalence classes from equivalencies.
pub fn partition_equivs<I, A>(coll: I) -> Vec<HashSet<A>>
where
    I: IntoIterator<Item = (A, A)>,
    A: Hash + Eq,
{
    coll.into_iter()
        .fold(vec![], |mut joined: Vec<HashSet<A>>, (l, r)| {
            let cycle = match joined.iter_mut().find(|c| c.contains(&l) || c.contains(&r)) {
                Some(c) => c,
                None => {
                    let idx = joined.len();
                    joined.push(HashSet::new());
                    &mut joined[idx]
                }
            };
            cycle.insert(l);
            cycle.insert(r);
            joined
        })
}

#[cfg(test)]
mod test {
    use super::*;
    fn unord_eq<A>(a: Vec<A>, mut b: Vec<A>) -> bool
    where
        A: Eq,
    {
        for x in a {
            match b.iter().enumerate().find_map(|(i, y)| (x == *y).then(|| i)) {
                Some(i) => b.remove(i),
                None => return false,
            };
        }
        b.is_empty()
    }

    #[test]
    fn test_partition_equivs() {
        let pairs = vec![(1, 2), (2, 3), (4, 3), (5, 6), (8, 8), (6, 7)];
        let classes = vec![hashset! {1, 2, 3, 4}, hashset! {5, 6, 7}, hashset! {8}];
        assert!(unord_eq(partition_equivs(pairs), classes));
    }

    #[test]
    fn test_canonical_pair() {
        assert_eq!((1, 2), canonical_pair(1, 2));
        assert_eq!((1, 2), canonical_pair(2, 1));
    }

    #[test]
    fn test_dot_var_cycles() {
        let dot_op: Term = opn!(Dot, var!("x"), str!("y"));
        let op = op!(
            And,
            opn!(Unify, dot_op.clone(), 1.into()),
            opn!(Unify, dot_op, var!("_this"))
        );

        // `x` and `_this` appear in the expn and a temporary will be
        // created for the output of the dot operation. check that
        // because the temporary is unified with `_this` the total
        // number of distinct variables in the output is 2.
        let vars = Vars::from(&op);
        assert_eq!(vars.variables.len(), 2);
    }
}
