use std::{
    collections::{HashMap, HashSet},
    fmt::{self, Display, Formatter},
    hash::Hash,
};

use crate::{
    data_filtering::{unregistered_field_error, unsupported_op_error, PartialResults, Type},
    error::{invalid_state_error, RuntimeError},
    events::ResultEvent,
    terms::*,
};

use serde::Serialize;

type FilterResult<A> = core::result::Result<A, RuntimeError>;

type TypeName = String;
type FieldName = String;
type VarName = String;

type Map<A, B> = HashMap<A, B>;
type Set<A> = HashSet<A>;

/// Represents an abstract filter over a data source.
///
/// `root` is a data type name supplied by the host, for example "User".
///
/// `relations` is a set of named logical extensions from the root data type to
/// other data types (representing "joins" for example).
///
/// `conditions` is is set of sets of binary relations (an OR of ANDs) that must
/// hold over the data source: for every record in the data source, if for some
/// top-level set in `conditions` every inner condition holds on the record, then
/// the record passes through the filter.
#[derive(Clone, Eq, Debug, Serialize, PartialEq)]
pub struct Filter {
    root: TypeName,                  // the host already has this, so we could leave it off
    relations: Set<Relation>,        // this & root determine the "joins" (or whatever)
    conditions: Vec<Set<Condition>>, // disjunctive normal form
}

/// A named logical extension of a data set. Corresponds to a "join" in relational
/// algebra, but we leave out the details about columns (the host knows how to do
/// it).
///
/// Fields represent "from", "through" and "to".
/// For example, Relation("Foo", "bar", "Bar") represents a Relation
/// from the `Foo` type to the `Bar` type, accessed using the `bar` field
/// on `Foo`.
#[derive(PartialEq, Eq, Debug, Serialize, Clone, Hash)]
pub struct Relation(TypeName, FieldName, TypeName);

/// A constraint that must hold for a record in the data source.
#[derive(PartialEq, Eq, Debug, Serialize, Clone, Hash)]
pub struct Condition(Datum, Comparison, Datum);

/// The left or right side of a Condition.
#[derive(PartialEq, Eq, Debug, Serialize, Clone, Hash)]
pub enum Datum {
    Field(Projection),
    Immediate(Value),
}

/// The comparison operation applied by a Condition.
#[derive(PartialEq, Debug, Serialize, Copy, Clone, Eq, Hash)]
pub enum Comparison {
    Eq,
    Neq,
    In,
}

/// An abstract "field reference" on a record from a named data source.
#[derive(PartialEq, Eq, Debug, Serialize, Clone, Hash)]
pub struct Projection(TypeName, Option<FieldName>);

type TypeInfo = Map<TypeName, Map<FieldName, Type>>;
type VarTypes = Map<PathVar, TypeName>;

/// Used to keep track of information for building a Filter
#[derive(Default)]
struct FilterInfo {
    type_info: TypeInfo,
    entities: VarTypes,
    conditions: Set<Condition>,
    relations: Set<Relation>,
}

/// A variable with zero or more "dot lookups"
///     a.b.c.d <-> PathVar { var: "a", path: ["b", "c", "d"] }
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct PathVar {
    var: VarName,
    path: Vec<FieldName>,
}

impl From<String> for PathVar {
    fn from(var: String) -> Self {
        Self { var, path: vec![] }
    }
}

impl From<Projection> for PathVar {
    fn from(Projection(var, field): Projection) -> Self {
        let path = field.into_iter().collect();
        PathVar { var, path }
    }
}

impl PathVar {
    fn from_term(t: &Term) -> FilterResult<Self> {
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

impl Filter {
    pub fn build(
        types: TypeInfo,
        ors: PartialResults,
        var: &str,
        class: &str,
    ) -> FilterResult<Self> {
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
        types: &TypeInfo,
        ands: ResultEvent,
        var: &Symbol,
        class: &str,
    ) -> FilterResult<Self> {
        ands.bindings
            .get(var)
            .map(|ands| Self::from_partial(types, ands, class))
            .unwrap_or_else(|| invalid_state_error(format!("unbound variable: {}", var.0)))
    }

    fn from_partial(types: &TypeInfo, ands: &Term, class: &str) -> FilterResult<Self> {
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
                .collect::<FilterResult<Vec<_>>>()
                .and_then(|ands| FilterInfo::build_filter(types.clone(), ands, class)),

            // sometimes we get an instance back. that means the variable
            // is exactly this instance, so return a filter that matches it.
            i @ ExternalInstance(_) => Ok(Filter {
                root: class.to_string(),
                relations: HashSet::new(),
                conditions: vec![singleton(Condition(
                    Field(Projection(class.to_string(), None)),
                    Comparison::Eq,
                    Immediate(i.clone()),
                ))],
            }),

            // oops, we don't know how to handle this!
            _ => invalid_state_error(ands.to_polar()),
        }
    }

    fn empty(class: &str) -> Self {
        use {Datum::Immediate, Value::Boolean};
        Self {
            root: class.to_string(),
            relations: Default::default(),
            conditions: vec![singleton(Condition(
                Immediate(Boolean(true)),
                Comparison::Eq,
                Immediate(Boolean(false)),
            ))],
        }
    }

    fn union(self, mut other: Self) -> Self {
        other.conditions.extend(self.conditions);
        other.relations.extend(self.relations);
        other
    }
}

impl FilterInfo {
    /// try to match a type and a field name with a relation
    fn get_relation_def(&mut self, typ: &str, dot: &str) -> Option<Relation> {
        if let Some(Type::Relation {
            other_class_tag, ..
        }) = self.type_info.get(typ).and_then(|map| map.get(dot))
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

    /// turn a pathvar into a projection.
    /// populates relations as a side effect
    fn pathvar2proj(&mut self, pv: PathVar) -> FilterResult<Projection> {
        let PathVar { mut path, var } = pv;
        // new var with empty path
        let mut pv = PathVar::from(var);
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
            match self.get_relation_def(&typ, &dot) {
                None => return unregistered_field_error(&typ, &dot),
                Some(rel) => {
                    let Relation(_, name, right) = &rel;
                    typ = right.clone();
                    pv.path.push(name.clone());
                    self.entities.insert(pv.clone(), right.clone());
                    self.relations.insert(rel);
                }
            }
        }

        // if the last path component names a relation from typ to typ'
        // then typ' is the new type and field is None. otherwise,
        // typ & field stay the same.
        match field
            .as_ref()
            .and_then(|dot| self.get_relation_def(&typ, dot))
        {
            None => Ok(Projection(typ, field)),
            Some(rel) => {
                let tag = rel.2.clone();
                pv.path.push(rel.1.clone());
                self.entities.insert(pv, tag.clone());
                self.relations.insert(rel);
                Ok(Projection(tag, None))
            }
        }
    }

    fn term2datum(&mut self, x: &Term) -> FilterResult<Datum> {
        use Datum::*;
        match PathVar::from_term(x) {
            Ok(pv) => Ok(Field(self.pathvar2proj(pv)?)),
            _ => Ok(Immediate(x.value().clone())),
        }
    }

    fn get_type(&mut self, pv: PathVar) -> Option<String> {
        self.entities.get(&pv).cloned().or_else(|| {
            let pv2 = pv.var.clone().into();
            let mut typ = self.entities.get(&pv2)?;
            for dot in pv.path.iter() {
                match self.type_info.get(typ)?.get(dot)? {
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
    fn add_constraint(&mut self, op: Operation) -> FilterResult<()> {
        use {Datum::*, Operator::*};
        let (left, right) = (self.term2datum(&op.args[0])?, self.term2datum(&op.args[1])?);
        match op.operator {
            Unify => self.add_eq_condition(left, right),
            Neq => self.add_neq_condition(left, right),
            In => match (&left, &right) {
                (Immediate(_), Field(Projection(_, None)))
                | (Field(Projection(_, None)), Field(Projection(_, None))) => {
                    self.add_eq_condition(left, right)
                }
                _ => self.add_in_condition(left, right),
            },
            _ => unsupported_op_error(op),
        }
    }

    fn add_condition(&mut self, l: Datum, op: Comparison, r: Datum) -> FilterResult<()> {
        self.conditions.insert(Condition(l, op, r));
        Ok(())
    }

    fn add_eq_condition(&mut self, left: Datum, right: Datum) -> FilterResult<()> {
        // only add condition if the side aren't == (otherwise it's redundant)
        if left != right {
            self.add_condition(left, Comparison::Eq, right)?;
        }
        Ok(())
    }

    fn add_neq_condition(&mut self, left: Datum, right: Datum) -> FilterResult<()> {
        self.add_condition(left, Comparison::Neq, right)
    }

    fn add_in_condition(&mut self, left: Datum, right: Datum) -> FilterResult<()> {
        self.add_condition(left, Comparison::In, right)
    }

    /// Validate FilterInfo before constructing a Filter
    fn validate(self, root: &str) -> FilterResult<Self> {
        let mut set = singleton(root);
        for Relation(_, _, dst) in self.relations.iter() {
            if set.contains(dst as &str) {
                return invalid_state_error(format!(
                    "Type `{}` occurs more than once as the target of a relation",
                    dst
                ));
            } else {
                set.insert(dst);
            }
        }
        Ok(self)
    }

    /// populate conditions and relations on an initialized FilterInfo
    fn with_constraints(mut self, ops: Set<Operation>, class: &str) -> FilterResult<Self> {
        // find pairs of implicitly equal variables
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

        // add every variable whose type we know to the entities map.
        //
        // partition variables into equivalence classes
        crate::data_filtering::partition_equivs(equivs)
            // map each variable to its own equivalence class
            .into_iter()
            .map(std::rc::Rc::new)
            .flat_map(|cls| {
                cls.iter()
                    .cloned()
                    .map(|pv| (pv, cls.clone()))
                    .collect::<Vec<_>>()
            })
            // for each variable k, if a variable in k's
            // eq class has a known type, then assign that
            // type to k.
            .filter_map(|(k, v)| {
                v.iter()
                    .find_map(|eq| self.get_type(eq.clone()))
                    .map(|t| (k, t))
            })
            .collect::<Vec<_>>() // so the closure ^^^ lets go of &mut self
            .into_iter()
            .for_each(|(k, t)| {
                // add them to the entities map
                self.entities.insert(k, t);
            });

        // every variable that needs a type
        // should now hopefully have a type.
        // now add a condition for each partial.
        // this also populates the relations.
        for op in ops {
            self.add_constraint(op)?;
        }

        self.validate(class)
    }

    fn build_filter(
        type_info: TypeInfo,
        parts: Vec<Operation>,
        class: &str,
    ) -> FilterResult<Filter> {
        let entities =
            std::iter::once((PathVar::from(String::from("_this")), class.to_string())).collect();

        // TODO(gw) check more isas in host -- rn we only check external instances
        let (_isas, othas): (Set<_>, Set<_>) = parts
            .into_iter()
            .partition(|op| op.operator == Operator::Isa);

        let Self {
            conditions,
            relations,
            ..
        } = Self {
            type_info,
            entities,
            ..Default::default()
        }
        .with_constraints(othas, class)?;

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

impl Display for Comparison {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        use Comparison::*;
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
            Immediate(val) => write!(f, "{}", val.to_polar()),
            Field(Projection(typ, None)) => write!(f, "{}", typ),
            Field(Projection(typ, Some(field))) => write!(f, "{}.{}", typ, field),
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

pub fn singleton<X>(x: X) -> Set<X>
where
    X: Hash + Eq,
{
    std::iter::once(x).collect()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_dup_reln() {
        let s = String::from;
        let types = hashmap! {
            s("Resource") => hashmap!{
                s("foo") => Type::Relation {
                   kind: s("one"),
                   my_field: s("_"),
                   other_field: s("_"),
                   other_class_tag: s("Foo")
                }
            },
            s("Foo") => hashmap!{
                s("y") => Type::Base {
                    class_tag: s("Integer")
                },
                s("resource") => Type::Relation {
                    kind: s("one"),
                    my_field: s("_"),
                    other_field: s("_"),
                    other_class_tag: s("Resource"),
                }
            }
        };

        // this is a great example of why i want to have shorter macros like opn! ptn! etc ...
        let ors = vec![ResultEvent::new(hashmap! {
            sym!("resource") => term!(op!(And,
                term!(op!(Isa, var!("_this"), term!(pattern!(instance!("Resource"))))),
                term!(op!(Isa, term!(op!(Dot, var!("_this"), str!("foo"))), term!(pattern!(instance!("Foo"))))),
                term!(op!(Isa, term!(op!(Dot, term!(op!(Dot, var!("_this"), str!("foo"))), str!("resource"))), term!(pattern!(instance!("Foo"))))),
                term!(op!(Unify, term!(1), term!(op!(Dot, term!(op!(Dot, term!(op!(Dot, var!("_this"), str!("foo"))), str!("resource"))), str!("foo")))))))
        })];

        match Filter::build(types, ors, "resource", "Resource") {
            Err(RuntimeError::InvalidState { msg })
                if &msg == "Type `Resource` occurs more than once as the target of a relation" => {}
            _ => panic!("unexpected"),
        }
    }

    #[test]
    fn test_in() {
        let types = hashmap! {
            String::from("Resource") => hashmap!{
                String::from("foos") => Type::Relation {
                   kind: String::from("many"),
                   my_field: String::from("_"),
                   other_field: String::from("_"),
                   other_class_tag: String::from("Foo")
                }
            },
            String::from("Foo") => hashmap!{
                String::from("y") => Type::Base {
                    class_tag: String::from("Integer")
                }
            }
        };

        let ors = vec![ResultEvent::new(hashmap! {
            sym!("resource") => term!(op!(And,
                term!(op!(Isa, var!("_this"), term!(pattern!(instance!("Resource"))))),
                term!(op!(In, var!("x"), term!(op!(Dot, var!("_this"), str!("foos"))))),
                term!(op!(Unify, term!(1), term!(op!(Dot, var!("x"), str!("y")))))
            ))
        })];

        let filter = Filter::build(types, ors, "resource", "Resource").unwrap();

        let Filter {
            root,
            relations,
            conditions,
        } = filter;

        assert_eq!(&root, "Resource");
        assert_eq!(
            relations,
            hashset! {
                Relation(String::from("Resource"), String::from("foos"), String::from("Foo"))
            }
        );

        assert_eq!(
            conditions,
            vec![hashset! {
                Condition(Datum::Immediate(value!(1)), Comparison::Eq, Datum::Field(Projection(String::from("Foo"), Some(String::from("y")))))
            }]
        );
    }
}
