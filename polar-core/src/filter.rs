use std::{
    collections::{HashMap, HashSet},
    fmt::{self, Display, Formatter},
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

#[derive(PartialEq, Eq, Debug, Serialize, Clone, Hash)]
pub struct Proj(TypeName, Option<FieldName>);

#[derive(PartialEq, Debug, Serialize, Copy, Clone, Eq, Hash)]
pub enum Compare {
    Eq,
    Neq,
    In,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct PathVar {
    var: String,
    path: Vec<String>,
}

impl From<String> for PathVar {
    fn from(var: String) -> Self {
        Self { var, path: vec![] }
    }
}

impl From<Proj> for PathVar {
    fn from(Proj(var, field): Proj) -> Self {
        let path = field.into_iter().collect();
        PathVar { var, path }
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
    pub fn build(types: Types, ors: PartialResults, var: &str, class: &str) -> PolarResult<Self> {
        let explain = std::env::var("POLAR_EXPLAIN").is_ok();

        if explain {
            eprintln!("\n===Data Filtering Query===");
            eprintln!("\n==Bindings==")
        }

        let var = Symbol(var.to_string());
        let filter = ors
            .into_iter()
            .map(|ands| Self::from_result_event(&types, ands, &var, class))
            .reduce(|l, r| Ok(l?.union(r?)))
            .unwrap_or_else(|| Ok(Self::empty(class)))?;

        if explain {
            eprintln!("\n==Filter==\n{}", filter);
        }

        Ok(filter)
    }

    fn from_result_event(
        types: &Types,
        ands: ResultEvent,
        var: &Symbol,
        class: &str,
    ) -> PolarResult<Self> {
        ands.bindings
            .get(var)
            .map(|ands| Self::from_partial(types, ands, class))
            .unwrap_or_else(|| input_error(format!("unbound variable: {}", var.0)))
    }

    fn from_partial(types: &Types, ands: &Term, class: &str) -> PolarResult<Self> {
        use {Datum::*, Operator::*, Value::*};

        if std::env::var("POLAR_EXPLAIN").is_ok() {
            eprintln!("{}", ands.to_polar());
        }

        match ands.value() {
            // most of the time we're dealing with expressions from the
            // simplifier.
            Expression(Operation {
                operator: And,
                args,
            }) => args
                .iter()
                .map(|and| Ok(and.value().as_expression()?.clone()))
                .collect::<PolarResult<Vec<_>>>()
                .and_then(|ands| QueryInfo::build_filter(types.clone(), ands, class)),

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
            _ => input_error(ands.to_polar()),
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

    /// turn a pathvar into a projection
    fn pathvar2proj(&mut self, pv: PathVar) -> PolarResult<Proj> {
        let PathVar { mut path, var } = pv;
        let mut pv = PathVar::from(var); // new var with empty path
                                         // what type is the base variable?
        let mut typ = match self.get_type(pv.clone()) {
            Some(typ) => typ,
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
                    pv.path.push(rel.1.clone());
                    self.entities.insert(pv.clone(), typ.clone());
                    self.relations.insert(rel);
                }
            }
        }

        // if the last path component names a relation from typ to typ'
        // then typ' is the new type and field is None. otherwise,
        // typ & field stay the same.
        let proj = match field.as_ref().and_then(|dot| self.get_relation(&typ, dot)) {
            None => Proj(typ, field),
            Some(rel) => {
                let tag = rel.2.clone();
                pv.path.push(rel.1.clone());
                self.entities.insert(pv, tag.clone());
                self.relations.insert(rel);
                Proj(tag, None)
            }
        };

        Ok(proj)
    }

    fn term2datum(&mut self, x: &Term) -> PolarResult<Datum> {
        use Datum::*;
        match PathVar::from_term(x) {
            Ok(pv) => Ok(Field(self.pathvar2proj(pv)?)),
            _ => Ok(Imm(x.value().clone())),
        }
    }

    fn add_condition(&mut self, l: Datum, op: Compare, r: Datum) -> PolarResult<()> {
        self.conditions.insert(Condition(l, op, r));
        Ok(())
    }

    fn get_type(&mut self, pv: PathVar) -> Option<String> {
        self.entities.get(&pv).cloned().or_else(|| {
            let pv2 = pv.var.clone().into();
            let mut typ = self.entities.get(&pv2)?;
            for dot in pv.path.iter() {
                match self.types.get(typ)?.get(dot)? {
                    Type::Relation {
                        other_class_tag, ..
                    } => typ = other_class_tag,
                    _ => return None,
                }
            }

            let typ = typ.clone();
            self.entities.insert(pv, typ.clone());
            Some(typ)
        })
    }

    /// digest a conjunct from the partial results & add a new constraint.
    fn add_constraint(&mut self, op: Operation) -> PolarResult<()> {
        use {Datum::*, Operator::*};
        let (left, right) = (self.term2datum(&op.args[0])?, self.term2datum(&op.args[1])?);
        match op.operator {
            Unify => self.add_condition(left, Compare::Eq, right),
            Neq => self.add_condition(left, Compare::Neq, right),
            In => match (&left, &right) {
                (Imm(_), Field(Proj(_, None))) | (Field(Proj(_, None)), Field(Proj(_, None))) => {
                    self.add_condition(left, Compare::Eq, right)
                }
                _ => self.add_condition(left, Compare::In, right),
            },
            _ => unsupported_op_error(op),
        }
    }

    fn with_constraints(mut self, ops: Set<Operation>) -> PolarResult<Self> {
        let equivs = ops.iter().filter_map(|Operation { operator, args }| {
            use Operator::*;
            let (l, r) = (
                PathVar::from_term(&args[0]).ok()?,
                PathVar::from_term(&args[1]).ok()?,
            );
            match operator {
                Unify | In => Some((l, r)),
                _ => None,
            }
        });

        crate::data_filtering::partition_equivs(equivs)
            .into_iter()
            .map(std::rc::Rc::new)
            .flat_map(|cls| {
                cls.iter()
                    .cloned()
                    .map(|pv| (pv, cls.clone()))
                    .collect::<Vec<_>>()
            })
            .filter_map(|(k, v)| {
                v.iter()
                    .find_map(|eq| self.get_type(eq.clone()))
                    .map(|t| (k, t))
            })
            .collect::<Vec<_>>()
            .into_iter()
            .for_each(|(k, t)| {
                self.entities.insert(k, t);
            });

        // each partial adds a constraint and may add relations
        for op in ops {
            self.add_constraint(op)?;
        }

        Ok(self)
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

        let Self {
            conditions,
            relations,
            ..
        } = Self {
            types,
            entities,
            ..Default::default()
        }.with_constraints(othas)?;


        Ok(Filter {
            relations,
            conditions: vec![conditions],
            root: class.to_string(),
        })
    }
}

impl Display for Filter {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        writeln!(f, "query {}", self.root)?;
        if !self.relations.is_empty() {
            writeln!(f, "join")?;
            for rel in &self.relations {
                writeln!(f, "    {}", rel)?;
            }
        }

        let mut disjs = self.conditions.iter();
        if let Some(disj) = disjs.next() {
            writeln!(f, "where")?;
            fmt_disj(disj, f)?;
            for disj in disjs {
                writeln!(f, "\n  OR")?;
                fmt_disj(disj, f)?;
            }
        }

        return Ok(());

        fn fmt_disj(disj: &Set<Condition>, f: &mut Formatter) -> Result<(), fmt::Error> {
            let mut conjs = disj.iter();
            match conjs.next() {
                None => {}
                Some(conj) => {
                    write!(f, "    {}", conj)?;
                    for conj in conjs {
                        write!(f, " AND\n    {}", conj)?;
                    }
                }
            }
            Ok(())
        }
    }
}

impl Display for Compare {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        use Compare::*;
        write!(
            f,
            "{}",
            match self {
                Eq => "=",
                Neq => "!=",
                In => "IN",
            }
        )
    }
}

impl Display for Datum {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        use Datum::*;
        match self {
            Imm(val) => write!(f, "{}", val.to_polar()),
            Field(Proj(typ, None)) => write!(f, "{}", typ),
            Field(Proj(typ, Some(field))) => write!(f, "{}.{}", typ, field),
        }
    }
}

impl Display for Condition {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        let Condition(l, op, r) = self;
        write!(f, "{} {} {}", l, op, r)
    }
}

impl Display for Relation {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        let Relation(src, nom, dest) = self;
        write!(f, "{}.{} -> {}", src, nom, dest)
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
