use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use crate::{
    data_filtering::{
        invalid_state_error, unregistered_field_error, unsupported_op_error, PartialResults, Type,
        Types,
    },
    error::{PolarResult, RuntimeError},
    events::ResultEvent,
    terms::*,
};

use serde::Serialize;

type TypeName = String;
type FieldName = String;
type Map<A, B> = HashMap<A, B>;
type Set<A> = HashSet<A>;

#[derive(PartialEq, Eq, Debug, Serialize, Clone, Hash)]
pub struct Proj(TypeName, Option<FieldName>);
#[derive(PartialEq, Eq, Debug, Serialize, Clone, Hash)]
pub enum Datum {
    Field(Proj),
    Imm(Value),
}

#[derive(PartialEq, Debug, Serialize, Copy, Clone, Eq, Hash)]
pub enum Compare {
    Eq,
    Neq,
    In,
}

#[derive(PartialEq, Eq, Debug, Serialize, Clone, Hash)]
pub struct Condition(Datum, Compare, Datum);
#[derive(PartialEq, Eq, Debug, Serialize, Clone, Hash)]
pub struct Relation(TypeName, FieldName, TypeName);

#[derive(Clone, Eq, Debug, Serialize, PartialEq)]
pub struct Filter {
    root: TypeName, // the host already has this, so we could leave it off
    relations: Set<Relation>,
    conditions: Vec<Set<Condition>>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash)]
pub struct PathVar {
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

trait Sources {
    fn sources(&self) -> Set<String>;
}

impl Sources for Proj {
    fn sources(&self) -> Set<TypeName> {
        singleton(self.0.clone())
    }
}

impl Sources for Datum {
    fn sources(&self) -> Set<String> {
        match self {
            Self::Field(proj) => proj.sources(),
            _ => HashSet::new(),
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
        let var = Symbol(var.to_string());
        disjuncts
            .into_iter()
            .map(|part| Self::from_result_event(&types, part, &var, class))
            .reduce(|left, right| Ok(left?.union(right?)))
            .unwrap_or_else(|| Ok(Self::empty(class)))
    }

    fn from_result_event(
        types: &Types,
        part: ResultEvent,
        var: &Symbol,
        class: &str,
    ) -> PolarResult<Self> {
        use RuntimeError::IncompatibleBindings;
        part.bindings
            .get(var)
            .ok_or_else(|| IncompatibleBindings { msg: var.0.clone() }.into())
            .and_then(|part| Self::from_partial(types, part, class))
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
                .map(|arg| match arg.value().as_expression() {
                    Ok(x) => Ok(x.clone()),
                    Err(_) => input_error(arg.to_polar()),
                })
                .collect::<PolarResult<Vec<Operation>>>()
                .and_then(|args| QueryInfo::build_filter(types.clone(), args, class)),

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
            relations: HashSet::new(),
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
}

#[derive(Debug, Default)]
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
            None => pv.var, // FIXME(gw) evil hack, happens to make it work
            Some(c) => c.to_string(),
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
        Ok(
            match field.as_ref().and_then(|dot| self.get_relation(&typ, dot)) {
                None => Proj(typ, field),
                Some(rel) => {
                    typ = rel.2.clone();
                    self.relations.insert(rel);
                    Proj(typ, None)
                }
            },
        )
    }

    fn term2datum(&mut self, x: &Term) -> Datum {
        PathVar::from_term(x)
            .and_then(|pv| self.pathvar2proj(pv))
            .map_or_else(|_| Datum::Imm(x.value().clone()), Datum::Field)
    }

    /// digest a conjunct from the partial results & add a new constraint.
    fn add_constraint(&mut self, op: Operation) -> PolarResult<()> {
        let cmp = match op.operator {
            Operator::Unify => Compare::Eq,
            Operator::Neq => Compare::Neq,
            Operator::In => Compare::In,
            _ => return unsupported_op_error(op),
        };

        let (left, right) = (self.term2datum(&op.args[0]), self.term2datum(&op.args[1]));

        self.conditions.insert(Condition(left, cmp, right));
        Ok(())
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
    let mut set = HashSet::new();
    set.insert(x);
    set
}
