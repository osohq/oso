use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::error::{OperationalError, PolarResult};
use crate::events::ResultEvent;

use crate::counter::*;
use crate::terms::*;
use std::hash::Hash;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum Type {
    Base {
        class_tag: String,
    },
    Relation {
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

struct ResultSetBuilder<'a> {
    result_set: ResultSet,
    types: &'a Types,
    vars: &'a Vars,
    seen: HashSet<Id>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default)]
pub struct FilterPlan {
    result_sets: Vec<ResultSet>,
}

pub type Types = HashMap<String, HashMap<String, Type>>;
pub type PartialResults = Vec<ResultEvent>;

#[derive(Debug)]
struct VarInfo<'a> {
    cycles: Vec<(Symbol, Symbol)>,                      // x = y
    uncycles: Vec<(Symbol, Symbol)>,                    // x != y
    types: Vec<(Symbol, String)>,                       // x matches XClass
    eq_values: Vec<(Symbol, Term)>,                     // x = 1
    contained_values: Vec<(Term, Symbol)>,              // 1 in x
    field_relationships: Vec<(Symbol, String, Symbol)>, // x.a = y
    in_relationships: Vec<(Symbol, Symbol)>,            // x in y
    counter: Counter,
    fields: &'a Types,
}

#[derive(Debug)]
struct Vars {
    variables: HashMap<Id, HashSet<Symbol>>,
    field_relationships: HashMap<Id, HashSet<(String, Id)>>,
    uncycles: HashMap<Id, HashSet<Id>>,
    in_relationships: HashSet<(Id, Id)>,
    eq_values: HashMap<Id, Term>,
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
    FilterPlan::build(types, partial_results, variable, class_tag)
}

impl From<Term> for Constraint {
    fn from(term: Term) -> Self {
        Self {
            kind: ConstraintKind::Eq,
            field: None,
            value: ConstraintValue::Term(term),
        }
    }
}

impl<'a> VarInfo<'a> {

    fn field_type(&self, base: &Symbol, field: &str) -> Option<&Type> {
        self.types.iter().find_map(|(s, t)| (s == base).then(|| t))
            .and_then(|tag| {
                self.fields.get(tag).and_then(|sub| {
                    sub.get(field)
                })
            })
    }

    fn from_op(op: &Operation, fields: &'a Types) -> PolarResult<Self> {
        Self {
            fields,
            cycles: vec![],
            uncycles: vec![],
            types: vec![],
            eq_values: vec![],
            contained_values: vec![],
            field_relationships: vec![],
            in_relationships: vec![],
            counter: Counter::default(),
        }.process_exp(op)
    }

    /// for when you absolutely, definitely need a symbol.
    fn symbolize(&mut self, val: &Term) -> Symbol {
        match val.value() {
            Value::Variable(var) | Value::RestVariable(var) =>
                var.clone(),
            Value::Expression(Operation { operator: Operator::Dot, args, }) =>
                self.dot_var(&args[0], &args[1]),
            _ => match self.eq_values.iter().find_map(|(x, y)| (y == val).then(|| x)) {
                Some(var) => var.clone(),
                _ => {
                    let new_var = sym!(&format!("_sym_{}", self.counter.next()));
                    self.eq_values.push((new_var.clone(), val.clone()));
                    new_var
                }
            }
        }
    }

    /// convert a binary dot expression into a symbol.
    fn dot_var(&mut self, base: &Term, field: &Term) -> Symbol {
        // handle nested dot ops.
        let sym = self.symbolize(base);
        let field_str = field.value().as_string().unwrap();


        match self.field_relationships.iter().find_map(|(p, f, c)| (*p == sym && f == field_str).then(|| c)) {
            Some(var) => var.clone(),
            _ => {
                let new_var = sym!(&format!(
                    "_{}_dot_{}_{}",
                    sym.0,
                    field_str,
                    self.counter.next()
                ));

                /*
                match self.field_type(&sym, field_str) {
                    Some(Type::Base { class_tag }) =>
                        self.types.push((new_var.clone(), class_tag.clone())),
                    Some(Type::Relation { other_class_tag, kind, .. }) if kind == "one" =>
                        self.types.push((new_var.clone(), other_class_tag.clone())),
                    _ => (),
                };
                */

                // Record the relationship between the vars.
                self.field_relationships
                    .push((sym, field_str.to_string(), new_var.clone()));

                new_var
            }
        }
    }

    /// turn dot expressions into symbols but leave other things unchanged.
    fn undot(&mut self, term: &Term) -> Value {
        let val = term.value();
        match val.as_expression() {
            Ok(Operation {
                operator: Operator::Dot,
                args,
            }) if args.len() == 2 => Value::from(self.dot_var(&args[0], &args[1])),
            _ => val.clone(),
        }
    }

    fn do_and(self, args: &[Term]) -> PolarResult<Self> {
        args.iter().fold(Ok(self), |s, arg| s.and_then(|s| {
            let inner_exp = arg.value().as_expression().unwrap();
            s.process_exp(inner_exp)
        }))
    }

    fn do_dot(mut self, lhs: &Term, rhs: &Term) -> PolarResult<Self> {
        self.dot_var(lhs, rhs);
        Ok(self)
    }

    fn do_isa(mut self, lhs: &Term, rhs: &Term) -> PolarResult<Self> {
        match rhs.value().as_pattern() {
            Ok(Pattern::Instance(i)) if i.fields.fields.is_empty() => {
                let lhs = self.symbolize(lhs);
                self.types.push((lhs, i.tag.0.clone()));
                Ok(self)
            }
            _ => err_unimplemented(format!("Unsupported specializer: {}", rhs.to_polar())),
        }
    }

    fn do_unify(mut self, left: &Term, right: &Term) -> PolarResult<Self> {
        match (self.undot(left), self.undot(right)) {
            (Value::Variable(l), Value::Variable(r)) => {
                self.cycles.push((l, r));
                Ok(self)
            }
            (Value::Variable(var), val) | (val, Value::Variable(var)) => {
                self.eq_values.push((var, Term::from(val)));
                Ok(self)
            }
            // Unifying something else.
            // 1 = 1 is irrelevant for data filtering, other stuff seems like an error.
            // @NOTE(steve): Going with the same not yet supported message but if this is
            // coming through it's probably a bug in the simplifier.
            _ => err_unimplemented(format!(
                "Unsupported unification: {} = {}",
                left.to_polar(),
                right.to_polar()
            )),
        }
    }

    fn do_neq(mut self, left: &Term, right: &Term) -> PolarResult<Self> {
        match (self.undot(left), self.undot(right)) {
            (Value::Variable(l), Value::Variable(r)) => {
                self.uncycles.push((l, r));
                Ok(self)
            }
            (Value::Variable(l), _) => {
                let r = self.symbolize(right);
                self.uncycles.push((l, r));
                Ok(self)
            }
            (_, Value::Variable(r)) => {
                let l = self.symbolize(left);
                self.uncycles.push((l, r));
                Ok(self)
            }
            _ => err_unimplemented(format!(
                "Unsupported comparison: {} != {}",
                left.to_polar(),
                right.to_polar()
            )),
        }
    }

    fn do_in(mut self, left: &Term, right: &Term) -> PolarResult<Self> {
        match (self.undot(left), self.undot(right)) {
            (Value::Variable(l), Value::Variable(r)) => {
                self.in_relationships.push((l, r));
                Ok(self)
            }
            (val, Value::Variable(var)) => {
                self.contained_values.push((Term::from(val), var));
                Ok(self)
            }
            _ => err_unimplemented(format!(
                    "Unsupported `in` check: {} in {}",
                    left.to_polar(),
                    right.to_polar()
                )),
        }
    }

    /// Process an expression in the context of this VarInfo. Just does side effects.
    fn process_exp(self, exp: &Operation) -> PolarResult<Self> {
        let args = &exp.args;
        match exp.operator {
            Operator::And => self.do_and(args),
            Operator::Dot if args.len() == 2 => self.do_dot(&args[0], &args[1]),
            Operator::Isa if args.len() == 2 => self.do_isa(&args[0], &args[1]),
            Operator::Neq if args.len() == 2 => self.do_neq(&args[0], &args[1]),
            Operator::In if args.len() == 2 => self.do_in(&args[0], &args[1]),
            Operator::Unify | Operator::Eq | Operator::Assign if args.len() == 2 =>
                self.do_unify(&args[0], &args[1]),

            _ => err_unimplemented(format!(
                "the expression `{}` is not supported for data filtering",
                exp.to_polar()
            )),
        }
    }
}

fn err_invalid<A>(msg: String) -> PolarResult<A> {
    Err(OperationalError::InvalidState { msg }.into())
}

fn err_unimplemented<A>(msg: String) -> PolarResult<A> {
    Err(OperationalError::Unimplemented { msg }.into())
}

impl FilterPlan {
    fn build(
        types: Types,
        partial_results: PartialResults,
        var: &str,
        class_tag: &str,
    ) -> PolarResult<FilterPlan> {
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
                result.bindings.get(&Symbol::new(var)).map(|term| {
                    match term.value().as_expression() {
                        Ok(exp) if exp.operator == Operator::And => {
                            let vars = Vars::from_op(exp, &types)?;
                            if explain {
                                eprintln!("  {}: {}", i, term.to_polar());
                                vars.explain()
                            }

                            ResultSet::build(&types, &vars, class_tag)
                        }
                        _ => Ok(ResultSet::from((term.clone(), class_tag))),
                    }
                })
            })
            .collect::<PolarResult<Vec<ResultSet>>>()?;

        Ok(FilterPlan { result_sets }.optimize(explain))
    }

    fn optimize(self, explain: bool) -> Self {
        if explain {
            eprintln!("== Raw Filter Plan ==");
            self.explain();
            eprintln!("\nOptimizing...")
        }
        self.opt_pass(explain)
    }

    fn opt_pass(mut self, explain: bool) -> Self {

        // Remove duplicate result set in a union.
        let drop_plan = self.result_sets.iter().enumerate().find_map(|(i, rs1)| {
            self.result_sets
                .iter()
                .enumerate()
                .find_map(|(j, rs2)| (i != j && rs1 == rs2).then(|| j))
        });

        match drop_plan {
            None => self.opt_fin(explain),
            Some(plan_id) => {
                if explain {
                    eprintln!("* Removed duplicate result set.")
                }
                self.result_sets.remove(plan_id);
                self.opt_pass(explain)
            }
        }
        // Possible future optimization ideas.
        // * If two result sets are almost the same except for a single fetch
        //   that only has a single field check and the field is different, we
        //   can merge the two result sets and turn the field check into an `in`.
        //   This is basically "un-expanding" either an `in` or and `or` from the policy.
        //   This could be hard to find.
    }

    fn opt_fin(self, explain: bool) -> Self {
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

impl From<(Term, &str)> for ResultSet {
    fn from(pair: (Term, &str)) -> Self {
        let (term, tag) = pair;
        let fetch = FetchRequest {
            class_tag: tag.to_owned(),
            constraints: vec![term.into()],
        };
        let result_id = 0;
        let resolve_order = vec![result_id];

        let mut requests = HashMap::new();
        requests.insert(result_id, fetch);

        Self { resolve_order, result_id, requests }
    }
}

impl ResultSet {
    fn build(types: &Types, vars: &Vars, this_type: &str) -> PolarResult<Self> {
        let result_set = ResultSet {
            requests: HashMap::new(),
            resolve_order: vec![],
            result_id: vars.this_id,
        };

        let mut result_set_builder = ResultSetBuilder {
            result_set,
            types,
            vars,
            seen: HashSet::new(),
        };

        result_set_builder.constrain_var(vars.this_id, this_type)?;
        result_set_builder.validate_and_finish()
    }

}

impl FetchRequest {
    fn constrain(&mut self, kind: ConstraintKind, field: Option<String>, value: ConstraintValue) {
        self.constraints.push(Constraint { kind, field, value });
    }

    fn deps(&self) -> Vec<Id> {
        self.constraints.iter().filter_map(|c| match c.value {
            ConstraintValue::Ref(Ref { result_id, .. }) => Some(result_id),
            _ => None,
        }).collect()
    }
}

impl<'a> ResultSetBuilder<'a> {

    fn validate_and_finish(self) -> PolarResult<ResultSet> {

        let mut rset = self.result_set;
        for (i, rid1) in rset.resolve_order.iter().enumerate() {
            let mut ro = rset.resolve_order[..i].iter();
            let req = rset.requests.get_mut(&rid1).unwrap();
            req.constraints.retain(|c| {
                if let ConstraintValue::Ref(Ref { result_id: rid2, .. }) = c.value {
                    ro.any(|x| *x == rid2)
                } else {
                    true
                }
            })
        }
        let order = &rset.resolve_order;

        // error messages
        let missing = |id| err_invalid(format!("Request {} missing from resolve order {:?}", id, order));
        let bad_order = |id1, id2, rset|
            err_invalid(format!( "Result set {} is resolved before its dependency {} in {:?}", id1, id2, rset));

        for (id1, v) in rset.requests.iter() {
            match index_of(order, id1) {
                None => return missing(*id1),
                Some(idx1) => for id2 in v.deps() {
                    match index_of(order, &id2) {
                        None => return missing(id2),
                        Some(idx2) if idx2 >= idx1 =>
                            return bad_order(id1, id2, &rset),
                        _ => (),
                    }
                }
            }
        }
        Ok(rset)
    }

    fn constrain_var(&mut self, id: Id, var_type: &str) -> PolarResult<&mut Self> {
        if self.seen.insert(id) {
            // add a fetch request
            self.result_set.requests.insert(id, FetchRequest {
                class_tag: var_type.to_string(),
                constraints: vec![],
            });

            // apply constraints to this request
            self.constrain_fields(id, var_type)?
                .constrain_in_vars(id, var_type)?
                .constrain_eq_vars(id)?
                .constrain_neq_vars(id)?;

            // constrain dependencies FIRST
            for dep in self.result_set.requests.get(&id).unwrap().deps() {
                self.try_constrain_var(dep)?;
            }

            // THEN add this request to resolve_order
            self.result_set.resolve_order.push(id);
        }
        Ok(self)
    }

    fn constrain_neq_vars(&mut self, id: Id) -> PolarResult<&mut Self> {
        let mut request = self.result_set.requests.remove(&id).unwrap();
        if let Some(un) = self.vars.uncycles.get(&id) {
            for v in un {
                let value = if let Some(val) = self.vars.eq_values.get(v) {
                    ConstraintValue::Term(val.clone())
                } else {
                    ConstraintValue::Ref(Ref { field: None, result_id: *v })
                };
                request.constrain(ConstraintKind::Neq, None, value)
            }
        }
        self.result_set.requests.insert(id, request);
        Ok(self)
    }

    fn constrain_eq_vars(&mut self, id: Id) -> PolarResult<&mut Self> {
        if let Some(l) = self.vars.eq_values.get(&id) {
            let mut request = self.result_set.requests.remove(&id).unwrap();
            let val = ConstraintValue::Term(l.clone());
            request.constrain(ConstraintKind::Eq, None, val);
            self.result_set.requests.insert(id, request);
        }
        Ok(self)
    }

    fn constrain_in_vars(
        &mut self,
        id: Id,
        var_type: &str,
    ) -> PolarResult<&mut Self> {
        let mut request = self.result_set.requests.remove(&id).unwrap();
        // Constrain any vars that are `in` this var.
        // Add their constraints to this one.
        // @NOTE(steve): I think this is right, but I'm not totally sure.
        // This might assume that the current var is a relationship of kind "many".
        for l in self
            .vars
            .in_relationships
            .iter()
            .filter_map(|(l, r)| (*r == id).then(|| l))
        {
            self.constrain_var(*l, var_type)?;
            if let Some(in_result_set) = self.result_set.requests.get(l) {
                request.constraints.extend(in_result_set.constraints.clone());
            }
        }

        if let Some(vs) = self.vars.contained_values.get(&id) {
            for l in vs {
                request.constrain(
                    ConstraintKind::Eq,
                    None,
                    ConstraintValue::Term(l.clone()));
            }
        }

        self.result_set.requests.insert(id, request);
        Ok(self)
    }

    fn constrain_relation(
        &mut self,
        me: Id,
        child: Id,
        other_class_tag: &str,
        my_field: &str,
        other_field: &str,
    ) -> PolarResult<&mut Self> {
        let mut request = self.result_set.requests.remove(&me).unwrap();
        self.constrain_var(child, other_class_tag)?;

        // FIXME this doesn't work right!
        // If the constrained child var doesn't have any constraints on it, we don't need to
        // constrain this var. Otherwise we're just saying field foo in all Foos which
        // would fetch all Foos and not be good.
        // if let Some(child_result) = self.result_set.requests.remove(&child) {
        //    if child_result.constraints.is_empty() {
        //        self.result_set.requests.insert(child, child_result);
        //    } else {
        //        self.result_set.requests.insert(child, child_result);
        request.constrain(
            ConstraintKind::In,
            Some(my_field.to_string()),
            ConstraintValue::Ref(Ref {
                field: Some(other_field.to_string()),
                result_id: child,
            }));
        //    }
        //}

        self.result_set.requests.insert(me, request);
        Ok(self)
    }

    fn constrain_field_eq(
        &mut self,
        me: Id,
        field: &str,
        child: Id,
    ) -> PolarResult<&mut Self> {
        let mut request = self.result_set.requests.remove(&me).unwrap();
        self.vars.eq_values.get(&child).into_iter().for_each(|value| {
            request.constrain(
                ConstraintKind::Eq,
                Some(field.to_string()),
                ConstraintValue::Term(value.clone()));
        });
        self.result_set.requests.insert(me, request);
        Ok(self)
    }

    fn constrain_field_neq(
        &mut self,
        me: Id,
        field: &str,
        child: Id,
    ) -> PolarResult<&mut Self> {
        let mut request = self.result_set.requests.remove(&me).unwrap();
        if let Some(un) = self.vars.uncycles.get(&child) {
            for k in un.iter() {
                match (self.vars.eq_values.get(k), self.vars.eq_values.get(&child)) {
                    (Some(val), None) => {
                        request.constrain(
                            ConstraintKind::Neq,
                            Some(field.to_string()),
                            ConstraintValue::Term(val.clone()));
                    }
                    (None, None) => {
                        request.constrain(
                            ConstraintKind::Neq,
                            Some(field.to_string()),
                            ConstraintValue::Ref(Ref { result_id: *k, field: None }));
                    }
                    _ => (),
                }
            }
        }
        self.result_set.requests.insert(me, request);
        Ok(self)
    }

    fn constrain_field_contained(
        &mut self,
        me: Id,
        field: &str,
        child: Id,
    ) -> PolarResult<&mut Self> {
        let mut request = self.result_set.requests.remove(&me).unwrap();
        if let Some(vs) = self.vars.contained_values.get(&child) {
            for v in vs {
                request.constrain(
                    ConstraintKind::Contains,
                    Some(field.to_string()),
                    ConstraintValue::Term(v.clone()));
            }
        }
        self.result_set.requests.insert(me, request);
        Ok(self)
    }

    fn constrain_field_others_with_same_parent(
        &mut self,
        me: Id,
        my_field: &str,
        my_child: Id,
    ) -> PolarResult<&mut Self> {
        let mut request = self.result_set.requests.remove(&me).unwrap();
        if let Some(fs) = self.vars.field_relationships.get(&me) {
            for (f, c) in fs.iter().filter(|(f, _)| *f != my_field) {
                let my_field = Some(my_field.to_string());
                let value = ConstraintValue::Field(f.clone());
                if c == &my_child {
                    request.constrain(ConstraintKind::Eq, my_field, value);
                } else if let Some(un) = self.vars.uncycles.get(c) {
                    if un.contains(&my_child) {
                        request.constrain(ConstraintKind::Neq, my_field, value);
                    }
                }
            }
        }
        self.result_set.requests.insert(me, request);
        Ok(self)
    }

    fn constrain_field_others(
        &mut self,
        me: Id,
        my_field: &str,
        my_child: Id,
    ) -> PolarResult<&mut Self> {
        let mut request = self.result_set.requests.remove(&me).unwrap();
        let others = self.vars.field_relationships.iter().filter(|(k, _)| **k != me);
        for (other_parent, other_children) in others {
            for (other_field, other_child) in other_children {
                let my_field = Some(my_field.to_string());
                let value = ConstraintValue::Ref(Ref {
                    field: Some(other_field.clone()),
                    result_id: *other_parent
                });
                if *other_child == my_child {
                    request.constrain(ConstraintKind::Eq, my_field, value);
                } else if let Some(un) = self.vars.uncycles.get(other_child) {
                    if un.contains(&my_child) {
                        request.constrain(ConstraintKind::Neq, my_field, value);
                    }
                }
            }
        }
        self.result_set.requests.insert(me, request);
        Ok(self)
    }


    fn try_constrain_var(&mut self, id: Id) -> PolarResult<&mut Self> {
        if let Some(class_tag) = self.vars.types.get(&id) {
            self.constrain_var(id, class_tag)
        } else {
            eprintln!("can't constrain var {}", id);
            Ok(self)
        }
    }

    fn constrain_fields(
        &mut self,
        id: Id,
        var_type: &str,
    ) -> PolarResult<&mut Self> {
        match self.vars.field_relationships.get(&id) {
            None => Ok(self),
            Some(fs) => fs.iter().fold(Ok(self), |s, (field, child)| s.and_then(|me| {
                match me.types.get(var_type).and_then(|m| m.get(field)) {
                    Some(Type::Relation { other_class_tag, my_field, other_field, ..  }) =>
                        me.constrain_relation(id, *child, other_class_tag, my_field, other_field),
                    _ =>
                        me.constrain_field_eq(id, field, *child)?
                          .constrain_field_neq(id, field, *child)?
                          .constrain_field_contained(id, field, *child)?
                          .constrain_field_others_with_same_parent(id, field, *child)?
                          .constrain_field_others(id, field, *child),
                }
            }))
        }
    }
}

impl Vars {
    fn from_op(op: &Operation, fields: &Types) -> PolarResult<Self> {
        Self::from_info(VarInfo::from_op(op, fields)?)
    }

    /// Collapses the var info that we obtained from walking the expressions.
    /// Track equivalence classes of variables and assign each one an id.
    fn from_info(info: VarInfo) -> PolarResult<Self> {
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
            .into_iter()
            .map(|c| (counter.next(), c))
            .collect::<HashMap<_, _>>();

        let fields = info.field_relationships;
        let mut assign_id = |item| get_var_id(&mut variables, item, &counter);


        let uncycles = info
            .uncycles
            .into_iter()
            .fold(HashMap::new(), |map, (a, b)| {
                let (a, b) = (assign_id(a), assign_id(b));
                hash_map_set_add(hash_map_set_add(map, a, b), b, a)
            });

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


        eprintln!("input types {:?}", &info.types);
        let types = info
            .types
            .into_iter()
            .map(|(var, typ)| (assign_id(var), typ))
            .collect::<HashMap<_, _>>();
        eprintln!("types {:?}", types);

        let contained_values =
            info.contained_values
                .into_iter()
                .fold(HashMap::new(), |map, (val, var)|
                    hash_map_set_add(map, assign_id(var), val));

        let field_relationships = fields
            .into_iter()
            .fold(HashMap::new(), |map, (p, f, c)|
                hash_map_set_add(map, assign_id(p), (f, assign_id(c))));

        match seek_var_id(&variables, &sym!("_this")) {
            None => err_invalid("No `_this` variable".to_string()),
            Some(this_id) => Ok(Vars {
                variables,
                uncycles,
                field_relationships,
                in_relationships,
                eq_values,
                contained_values,
                types,
                this_id,
            }),
        }
    }

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
                    Value::ExternalInstance(_) => "ExternalInstance",
                    Value::Call(_) => "Call",
                    Value::Variable(_) => "Variable",
                    Value::RestVariable(_) => "RestVariable",
                    Value::Expression(_) => "Expression",
                    Value::Pattern(_) => "Pattern",
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
        for (x, fs) in self.field_relationships.iter() {
            for (field, y) in fs.iter() {
                eprintln!("      {}.{} = {}", x, field, y);
            }
        }
        eprintln!("    in relationships");
        for (x, y) in &self.in_relationships {
            eprintln!("      {} in {}", x, y);
        }
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
                    joined.push(HashSet::new());
                    joined.last_mut().unwrap()
                }
            };
            cycle.insert(l);
            cycle.insert(r);
            joined
        })
}

fn index_of<A>(v: &[A], x: &A) -> Option<usize>
where
    A: PartialEq<A>,
{
    v.iter().enumerate().find_map(|(i, y)| (y == x).then(|| i))
}

fn hash_map_set_add<A, B>(
    mut map: HashMap<A, HashSet<B>>,
    a: A,
    b: B) -> HashMap<A, HashSet<B>>
where
    A: Eq + Hash,
    B: Eq + Hash

{
    map.entry(a).or_insert_with(HashSet::new).insert(b);
    map
}


#[cfg(test)]
mod test {
    use super::*;
    use crate::bindings::Bindings;
    type TestResult = PolarResult<()>;

    impl From<Bindings> for ResultEvent {
        fn from(bindings: Bindings) -> Self {
            ResultEvent { bindings }
        }
    }

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
    fn test_dot_plan() -> TestResult {
        let ins0: Term = ExternalInstance::from(0).into();
        let ins1: Term = ExternalInstance::from(1).into();
        let pat_a = term!(pattern!(instance!("A")));
        let pat_b = term!(pattern!(instance!("B")));
        let partial = term!(op!(
            And,
            term!(op!(Isa, var!("_this"), pat_a)),
            term!(op!(Isa, ins0.clone(), pat_b.clone())),
            term!(op!(Isa, ins1.clone(), pat_b)),
            term!(op!(
                Unify,
                term!(op!(Dot, ins0, str!("field"))),
                var!("_this")
            )),
            term!(op!(
                Unify,
                term!(op!(Dot, var!("_this"), str!("field"))),
                ins1
            ))
        ));

        let bindings = ResultEvent::from(hashmap! {
            sym!("resource") => partial
        });

        let types = hashmap! {
            "A".to_owned() => hashmap! {
                "field".to_owned() => Type::Base {
                    class_tag: "B".to_owned()
                }
            },
            "B".to_owned() => hashmap! {
                "field".to_owned() => Type::Base {
                    class_tag: "A".to_owned()
                }
            }
        };
        build_filter_plan(types, vec![bindings], "resource", "something")?;
        Ok(())
    }

    #[test]
    fn test_empty_in() -> TestResult {
        let partial = term!(op!(And, term!(op!(In, var!("_this"), var!("x")))));
        let bindings = ResultEvent::from(hashmap! {
            sym!("resource") => partial
        });

        build_filter_plan(hashmap!{}, vec![bindings], "resource", "something")?;
        Ok(())
    }


    #[test]
    fn test_partition_equivs() {
        let pairs = vec![(1, 2), (2, 3), (4, 3), (5, 6), (8, 8), (6, 7)];
        let classes = vec![hashset! {1, 2, 3, 4}, hashset! {5, 6, 7}, hashset! {8}];
        assert!(unord_eq(partition_equivs(pairs), classes));
    }

    #[test]
    fn test_dot_var_cycles() -> PolarResult<()> {
        let dot_op: Term = term!(op!(Dot, var!("x"), str!("y")));
        let op = op!(
            And,
            term!(op!(Unify, dot_op.clone(), 1.into())),
            term!(op!(Unify, dot_op, var!("_this")))
        );

        // `x` and `_this` appear in the expn and a temporary will be
        // created for the output of the dot operation. check that
        // because the temporary is unified with `_this` the total
        // number of distinct variables in the output is 2.
        let vars = Vars::from_op(&op)?;
        assert_eq!(vars.variables.len(), 2);
        Ok(())
    }
}
