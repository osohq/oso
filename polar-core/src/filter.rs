use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use crate::{
    data_filtering::{unregistered_field_error, unsupported_op_error, PartialResults, Type, Types},
    error::{invalid_state_error, PolarResult, RuntimeError},
    events::ResultEvent,
    terms::*,
};

use serde::Serialize;

type TypeName = String;
type FieldName = String;
type Map<A, B> = HashMap<A, B>;
type Set<A> = HashSet<A>;

#[derive(Clone, Eq, Debug, Serialize, PartialEq)]
pub struct Filter {
    root: TypeName,                  // the host already has this, so we could leave it off
    conditions: Vec<Set<Condition>>, // disjunctive normal form
    relations: Set<Relation>,
}

#[derive(PartialEq, Eq, Debug, Serialize, Clone, Hash)]
pub struct Relation(TypeName, FieldName, TypeName);

#[derive(PartialEq, Eq, Debug, Serialize, Clone, Hash)]
pub struct Condition(Datum, Compare, Datum);

#[derive(PartialEq, Eq, Debug, Serialize, Clone, Hash)]
pub enum Datum {
    Field(Proj),
    Imm(Value),
}

impl Datum {
    pub fn explain(&self) {
        match self {
            Datum::Field(proj) => {
                proj.explain();
            }
            Datum::Imm(value) => {
                eprint!("{:?}", value);
            }
        }
    }
}

#[derive(PartialEq, Eq, Debug, Serialize, Clone, Hash)]
pub struct Proj(TypeName, Option<FieldName>);

impl Proj {
    pub fn explain(&self) {
        let Proj(typename, field) = self;
        eprint!("{}", typename);
        if let Some(field) = field {
            eprint!(".{}", field);
        }
    }
}

#[derive(PartialEq, Debug, Serialize, Copy, Clone, Eq, Hash)]
pub enum Compare {
    Eq,
    Neq,
    In,
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct PathVar {
    var: String,
    path: Vec<String>,
}

impl From<String> for PathVar {
    fn from(var: String) -> Self {
        Self { var, path: vec![] }
    }
}

impl PathVar {
    fn from_term(t: &Term) -> PolarResult<Self> {
        use Value::*;
        match t.value() {
            Expression(Operation {
                operator: Operator::Dot,
                args,
            }) => {
                let dot = args[1].value().as_string()?.to_string();
                let mut pv = Self::from_term(&args[0])?;
                pv.path.push(dot);
                Ok(pv)
            }
            Variable(Symbol(var)) => Ok(var.clone().into()),
            _ => invalid_state_error(format!("PathVar::from_term({})", t.to_polar())),
        }
    }
}

impl Operation {
    /// turn an isa from the partial results into a pathvar -> type pair
    fn into_entity(self) -> Option<PolarResult<(PathVar, TypeName)>> {
        match self.args[1].value() {
            Value::Pattern(Pattern::Instance(InstanceLiteral { tag, fields }))
                if fields.is_empty() =>
            {
                // FIXME(gw) this is to work around a complicated simplifier
                // bug that causes external instances to sometimes be embedded
                // in partials. term2pathvar will fail if the base of the path
                // is an external instance and not a var. we can't check if the
                // isa is true from in here, so just drop it for now. the host
                // should check for these before building the filter.
                match PathVar::from_term(&self.args[0]) {
                    Err(_) => None,
                    Ok(p) => Some(Ok((p, tag.0.clone()))),
                }
            }
            _ => Some(unsupported_op_error(self)),
        }
    }
}

impl Filter {
    pub fn build(
        types: Types,
        disjuncts: PartialResults,
        var: &str,
        class: &str,
    ) -> PolarResult<Self> {
        let explain = std::env::var("POLAR_EXPLAIN").is_ok();

        if explain {
            eprintln!("\n===Data Filtering Query===");
            eprintln!("\n==Bindings==")
        }

        let var = Symbol(var.to_string());
        disjuncts
            .into_iter()
            .map(|disjunct| Self::from_result_event(&types, disjunct, &var, class))
            .reduce(|l, r| Ok(l?.union(r?)))
            .unwrap_or_else(|| Ok(Self::empty(class)))
            .map(|filter| {
                if explain {
                    eprintln!("\n==Filter==");
                    filter.explain();
                }
                filter
            })
    }

    fn from_result_event(
        types: &Types,
        part: ResultEvent,
        var: &Symbol,
        class: &str,
    ) -> PolarResult<Self> {
        let explain = std::env::var("POLAR_EXPLAIN").is_ok();
        part.bindings
            .get(var)
            .map(|part| {
                if explain {
                    eprintln!("{}", part.to_polar());
                }
                Self::from_partial(types, part, class)
            })
            .unwrap_or_else(|| input_error(format!("unbound variable: {}", var.0)))
    }

    fn from_partial(types: &Types, term: &Term, class: &str) -> PolarResult<Self> {
        use {Datum::*, Operator::*, Value::*};
        match term.value() {
            // most of the time we're dealing with expressions from the
            // simplifier.
            Expression(Operation {
                operator: And,
                args,
            }) => args
                .iter()
                .map(|arg| Ok(arg.value().as_expression()?.clone()))
                .collect::<PolarResult<Vec<_>>>()
                .and_then(|conjuncts| QueryInfo::build_filter(types.clone(), conjuncts, class)),

            // sometimes we get an instance back. that means the variable
            // is exactly this instance, so return a filter that matches it.
            i @ ExternalInstance(_) => Ok(Filter {
                root: class.to_string(),
                relations: HashSet::new(),
                conditions: vec![singleton(Condition(
                    Field(Proj(class.to_string(), None)),
                    Compare::Eq,
                    Imm(i.clone()),
                ))],
            }),

            // oops, we don't know how to handle this!
            _ => input_error(term.to_polar()),
        }
    }

    fn empty(class: &str) -> Self {
        use {Datum::Imm, Value::Boolean};
        Self {
            root: class.to_string(),
            relations: Default::default(),
            conditions: vec![singleton(Condition(
                Imm(Boolean(true)),
                Compare::Eq,
                Imm(Boolean(false)),
            ))],
        }
    }

    fn union(self, mut other: Self) -> Self {
        other.conditions.extend(self.conditions);
        other.relations.extend(self.relations);
        other
    }

    fn explain(&self) {
        eprintln!("query {}", self.root);
        eprintln!("join");
        for Relation(_, f, ot) in &self.relations {
            eprintln!(" {} using {}", ot, f);
        }
        eprintln!("where");
        for (i, s) in self.conditions.iter().enumerate() {
            if i != 0 {
                eprintln!("\n  OR");
            }
            for (i, Condition(l, op, r)) in s.iter().enumerate() {
                if i != 0 {
                    eprintln!(" AND");
                }
                eprint!("    ");
                l.explain();
                eprint!(" {:?} ", op);
                r.explain();
            }
        }
    }
}

#[derive(Default)]
struct QueryInfo {
    types: Types,
    entities: Map<PathVar, TypeName>,
    conditions: Set<Condition>,
    relations: Set<Relation>,
}

impl QueryInfo {
    /// try to match a type and a field name with a relation
    fn get_relation(&mut self, typ: &str, dot: &str) -> Option<Relation> {
        if let Some(Type::Relation {
            other_class_tag, ..
        }) = self.types.get(typ).and_then(|map| map.get(dot))
        {
            Some(Relation(
                typ.to_string(),
                dot.to_string(),
                other_class_tag.to_string(),
            ))
        } else {
            None
        }
    }

    /// turn a pathvar into a projection and a set of addl relations
    fn pathvar2proj(&mut self, pv: PathVar) -> PolarResult<Proj> {
        let PathVar { mut path, var } = pv;
        let pv = PathVar::from(var); // new var with empty path
                                     // what type is the base variable?
        let mut typ = match self.entities.get(&pv) {
            Some(c) => c.to_string(),
            _ => return invalid_state_error(format!("unknown type for `{}`", pv.var)),
        };

        // the last part of the path is always allowed not to be a relation.
        // pop it off for now & deal with it in a minute.
        let field = path.pop();

        // all the middle parts have to be relations, so if we can't find one
        // then we fail here.
        for dot in path {
            match self.get_relation(&typ, &dot) {
                None => return unregistered_field_error(&typ, &dot),
                Some(rel) => {
                    typ = rel.2.clone();
                    self.relations.insert(rel);
                }
            }
        }

        // if the last path component names a relation from typ to typ'
        // then typ' is the new type and field is None. otherwise,
        // typ & field stay the same.
        let (typ, field) = match field.as_ref().and_then(|dot| self.get_relation(&typ, dot)) {
            None => (typ, field),
            Some(rel) => {
                let tag = rel.2.clone();
                self.relations.insert(rel);
                (tag, None)
            }
        };

        Ok(Proj(typ, field))
    }

    fn term2datum(&mut self, x: &Term) -> PolarResult<Datum> {
        use {Datum::*, Value::*};
        PathVar::from_term(x)
            .and_then(|pv| self.pathvar2proj(pv))
            .map(Field)
            .or_else(|_| match x.value() {
                v @ String(_) | v @ Number(_) | v @ Boolean(_) | v @ ExternalInstance(_) => {
                    Ok(Imm(v.clone()))
                }
                _ => invalid_state_error(format!("invalid immediate value: {}", x.to_polar())),
            })
    }

    fn add_condition(&mut self, l: Datum, op: Compare, r: Datum) -> PolarResult<()> {
        self.conditions.insert(Condition(l, op, r));
        Ok(())
    }

    /// digest a conjunct from the partial results & add a new constraint.
    fn add_constraint(&mut self, op: Operation) -> PolarResult<()> {
        use Datum::*;
        let (left, right) = (self.term2datum(&op.args[0])?, self.term2datum(&op.args[1])?);
        match op.operator {
            Operator::Unify => self.add_condition(left, Compare::Eq, right),
            Operator::Neq => self.add_condition(left, Compare::Neq, right),
            Operator::In => match (&left, &right) {
                (Imm(_), Field(Proj(_, None))) | (Field(Proj(_, None)), Field(Proj(_, None))) => {
                    self.add_condition(left, Compare::Eq, right)
                }
                _ => self.add_condition(left, Compare::In, right),
            },
            _ => unsupported_op_error(op),
        }
    }

    fn build_filter(types: Types, parts: Vec<Operation>, class: &str) -> PolarResult<Filter> {
        // we use isa constraints to initialize the entities map
        let (isas, othas): (Set<_>, Set<_>) = parts
            .into_iter()
            .partition(|op| op.operator == Operator::Isa);

        // entities maps variable paths to types
        let entities = isas
            .into_iter()
            .filter_map(|op| op.into_entity())
            .collect::<PolarResult<_>>()?;

        // start with types & entities
        let mut this = Self {
            types,
            entities,
            ..Default::default()
        };

        // each partial adds a constraint and may add relations
        for op in othas {
            this.add_constraint(op)?;
        }

        let Self {
            conditions,
            relations,
            ..
        } = this;

        Ok(Filter {
            relations,
            conditions: vec![conditions],
            root: class.to_string(),
        })
    }
}

fn input_error<A>(msg: String) -> PolarResult<A> {
    Err(RuntimeError::Unsupported { msg }.into())
}

pub fn singleton<X>(x: X) -> Set<X>
where
    X: Hash + Eq,
{
    std::iter::once(x).collect()
}
