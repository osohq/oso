use std::{
    collections::{HashMap, HashSet},
    fmt::{self, Display, Formatter},
    hash::Hash,
};

use crate::{
    data_filtering::{unregistered_field_error, unsupported_op_error, PartialResults, Type},
    error::{invalid_state_error, RuntimeError},
    normalize::*,
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
    relations: Vec<Relation>,        // this & root determine the "joins" (or whatever)
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
    Nin,
    Lt,
    Leq,
    Gt,
    Geq,
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
            _ => invalid_state_error(format!("PathVar::from_term({})", t)),
        }
    }
}

impl Filter {
    pub fn build(
        types: TypeInfo,
        partials: PartialResults,
        var: &str,
        class: &str,
    ) -> FilterResult<Self> {
        let explain = std::env::var("POLAR_EXPLAIN").is_ok();

        if explain {
            eprintln!("\n===Data Filtering Query===");
            eprintln!("\n==Bindings==")
        }

        let sym = Symbol(var.to_string());
        let filter = partials
            .into_iter()
            .filter_map(|opt| opt.bindings.get(&sym).cloned())
            .reduce(or_)
            .into_iter()
            .flat_map(vec_of_ands)
            .map(|ands| Self::from_partial(&types, ands, var, class))
            .reduce(|l, r| Ok(l?.union(r?)))
            .unwrap_or_else(|| Ok(Self::empty(class)))?;

        if explain {
            eprintln!("\n==Filter==\n{}", filter);
        }

        Ok(filter)
    }

    fn from_partial(types: &TypeInfo, ands: Term, var: &str, class: &str) -> FilterResult<Self> {
        use {Operator::*, Value::*};

        if std::env::var("POLAR_EXPLAIN").is_ok() {
            eprintln!("{}", ands);
        }

        let term2expr = |i: Term| match i.value().as_expression() {
            Ok(x) => x.clone(),
            _ => op!(Unify, var!(var), i),
        };

        match ands.value() {
            // most of the time we're dealing with expressions from the
            // simplifier.
            Expression(Operation {
                operator: And,
                args,
            }) => args
                .iter()
                .map(|and| Ok(term2expr(and.clone())))
                .collect::<FilterResult<Vec<_>>>()
                .and_then(|ands| FilterInfo::build_filter(types.clone(), ands, var, class)),

            // sometimes we get an instance back. that means the variable
            // is exactly this instance, so return a filter that matches it.
            ExternalInstance(_) => {
                FilterInfo::build_filter(types.clone(), vec![term2expr(ands.clone())], var, class)
            }

            // oops, we don't know how to handle this!
            _ => invalid_state_error(ands.to_string()),
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

    fn union(mut self, other: Self) -> Self {
        self.conditions.extend(other.conditions);
        for rel in other.relations {
            if !self.relations.iter().any(|r| r == &rel) {
                self.relations.push(rel);
            }
        }
        self
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

    /// Digest a conjunct from the partial results & add a new constraint.
    fn add_constraint(&mut self, op: Operation) -> FilterResult<()> {
        match op.args.len() {
            1 => self.add_constraint_1(op),
            2 => self.add_constraint_2(op),
            _ => unsupported_op_error(op),
        }
    }

    /// Handle a unary operation from the simplifier
    fn add_constraint_1(&mut self, op: Operation) -> FilterResult<()> {
        use Operator::*;
        // The only case this currently handles is `not in`.
        match op.operator {
            Not => match op.args[0].value().as_expression() {
                Ok(Operation { operator: In, args }) if args.len() == 2 => {
                    let (left, right) = (self.term2datum(&args[0])?, self.term2datum(&args[1])?);
                    self.add_condition(left, Comparison::Nin, right)
                }
                _ => unsupported_op_error(op),
            },
            _ => unsupported_op_error(op),
        }
    }

    /// Handle a binary expression from the simplifier
    fn add_constraint_2(&mut self, op: Operation) -> FilterResult<()> {
        use {Datum::*, Operator::*};
        let (left, right) = (self.term2datum(&op.args[0])?, self.term2datum(&op.args[1])?);
        let op = match op.operator {
            Unify => Comparison::Eq,
            Neq => Comparison::Neq,
            In => match (&left, &right) {
                (Immediate(_), Field(Projection(_, None)))
                | (Field(Projection(_, None)), Field(Projection(_, None))) => Comparison::Eq,
                _ => Comparison::In,
            },
            Lt => Comparison::Lt,
            Leq => Comparison::Leq,
            Gt => Comparison::Gt,
            Geq => Comparison::Geq,
            _ => return unsupported_op_error(op),
        };
        self.add_condition(left, op, right)
    }

    fn add_condition(&mut self, left: Datum, op: Comparison, right: Datum) -> FilterResult<()> {
        use Comparison::*;
        match op {
            Eq | Leq | Geq if left == right => Ok(()),
            _ => {
                self.conditions.insert(Condition(left, op, right));
                Ok(())
            }
        }
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
        var: &str,
        class: &str,
    ) -> FilterResult<Filter> {
        fn sort_relations(
            relations: HashSet<Relation>,
            mut types: HashSet<TypeName>,
            mut out: Vec<Relation>,
        ) -> Vec<Relation> {
            if relations.is_empty() {
                return out;
            }
            let mut rest = HashSet::new();
            for rel in relations {
                if types.contains(&rel.0) {
                    types.insert(rel.2.clone());
                    out.push(rel);
                } else {
                    rest.insert(rel);
                }
            }
            sort_relations(rest, types, out)
        }

        // TODO(gw) check more isas in host -- rn we only check external instances
        let (_isas, othas): (Set<_>, Set<_>) = parts
            .into_iter()
            .partition(|op| op.operator == Operator::Isa);

        let mut entities = HashMap::new();
        entities.insert(PathVar::from("_this".to_string()), class.to_string());
        entities.insert(PathVar::from(var.to_string()), class.to_string());

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

        let relations = sort_relations(relations, singleton(class.to_string()), vec![]);

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
                Nin => "NOT IN",
                Lt => "<",
                Gt => ">",
                Leq => "<=",
                Geq => ">=",
            }
        )
    }
}

impl Display for Datum {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        use Datum::*;
        match self {
            Immediate(val) => write!(f, "{}", val),
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

fn vec_of_ands(t: Term) -> Vec<Term> {
    fn or_of_ands(t: Term) -> Vec<Term> {
        use Operator::*;
        match t.value().as_expression() {
            Ok(Operation { operator, args }) if *operator == Or => {
                args.iter().cloned().flat_map(or_of_ands).collect()
            }
            _ => {
                vec![term!(Operation {
                    operator: And,
                    args: ands(t),
                })]
            }
        }
    }

    fn ands(t: Term) -> Vec<Term> {
        use Operator::*;
        match t.value().as_expression() {
            Ok(Operation { operator, args }) if *operator == And => {
                args.iter().cloned().flat_map(ands).collect()
            }
            _ => vec![t],
        }
    }

    or_of_ands(t.disjunctive_normal_form())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::events::ResultEvent;

    type TestResult = Result<(), RuntimeError>;
    type TypeMap = Map<String, Map<String, Type>>;

    fn types_0() -> TypeMap {
        let s = String::from;
        hashmap! {
            s("Foo") => hashmap!{
                s("id") => Type::Base {
                    class_tag: s("Integer")
                },
            }
        }
    }

    #[test]
    fn test_or_normalization() -> TestResult {
        let types = types_0;

        // two conditions behind an `or` in one result
        let ex1 = vec![ResultEvent::new(hashmap! {
            sym!("resource") =>
                    term!(op!(Or,
                        term!(op!(Unify,
                            term!(op!(Dot, var!("_this"), str!("id"))),
                            term!(1))),
                        term!(op!(Unify,
                            term!(op!(Dot, var!("_this"), str!("id"))),
                            term!(2))))),
        })];

        // two results with one condition each
        let ex2 = vec![
            ResultEvent::new(hashmap! {
                sym!("resource") =>
                            term!(op!(Unify,
                                term!(op!(Dot, var!("_this"), str!("id"))),
                                term!(1))),
            }),
            ResultEvent::new(hashmap! {
                sym!("resource") =>
                            term!(op!(Unify,
                                term!(op!(Dot, var!("_this"), str!("id"))),
                                term!(2))),
            }),
        ];

        assert_eq!(
            Filter::build(types(), ex1, "resource", "Foo")?,
            Filter::build(types(), ex2, "resource", "Foo")?,
        );

        Ok(())
    }

    fn types_1() -> TypeMap {
        let s = String::from;
        hashmap! {
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
        }
    }

    #[test]
    fn test_dup_reln() {
        let types = types_1();

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
            x => panic!("unexpected: {:?}", x),
        }
    }

    #[test]
    fn test_in() -> TestResult {
        let s = String::from;
        let types = hashmap! {
            s("Resource") => hashmap!{
                s("foos") => Type::Relation {
                   kind: s("many"),
                   my_field: s("_"),
                   other_field: s("_"),
                   other_class_tag: s("Foo")
                }
            },
            s("Foo") => hashmap!{
                s("y") => Type::Base {
                    class_tag: s("Integer")
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

        let Filter {
            root,
            relations,
            conditions,
        } = Filter::build(types, ors, "resource", "Resource")?;

        assert_eq!(&root, "Resource");

        assert_eq!(
            relations,
            vec![Relation(s("Resource"), s("foos"), s("Foo"))]
        );

        assert_eq!(
            conditions,
            vec![hashset! {
                Condition(Datum::Immediate(value!(1)), Comparison::Eq, Datum::Field(Projection(String::from("Foo"), Some(String::from("y")))))
            }]
        );
        Ok(())
    }

    #[test]
    fn test_vec_of_ands() {
        let ex = or_(
            not_(var!("p")),
            and_(var!("q"), not_(and_(not_(var!("r")), var!("s")))),
        );

        let oa = vec![
            not_(var!("p")),
            and_(var!("q"), var!("r")),
            and_(var!("q"), not_(var!("s"))),
        ];

        let to_s =
            |ooa: Vec<Term>| format!("{:?}", ooa.iter().map(Term::to_string).collect::<Vec<_>>());

        assert_eq!(to_s(oa), to_s(vec_of_ands(ex)));
    }

    fn types_2() -> TypeMap {
        let s = String::from;
        hashmap! {
            s("Resource") => hashmap!{
                s("foo") => Type::Relation {
                   kind: s("one"),
                   my_field: s("_"),
                   other_field: s("_"),
                   other_class_tag: s("Foo")
                }
            },
            s("Foo") => hashmap!{
                s("boo") => Type::Relation {
                    kind: s("one"),
                    my_field: s("_"),
                    other_field: s("_"),
                    other_class_tag: s("Boo"),
                }
            },
            s("Boo") => hashmap!{
                s("goo") => Type::Relation {
                    kind: s("one"),
                    my_field: s("_"),
                    other_field: s("_"),
                    other_class_tag: s("Goo"),
                }
            },
            s("Goo") => hashmap!{
                s("id") => Type::Base {
                    class_tag: s("Integer")
                }
            }
        }
    }

    #[test]
    fn test_relation_depsort() -> TestResult {
        let s = String::from;
        let types = types_2();
        let ors = vec![ResultEvent::new(hashmap! {
            sym!("resource") => term!(op!(And,
                term!(op!(Isa, var!("_this"), term!(pattern!(instance!("Resource"))))),
                term!(op!(Unify, term!(1), term!(op!(Dot, term!(op!(Dot, term!(op!(Dot, term!(op!(Dot, var!("_this"), str!("foo"))), str!("boo"))), str!("goo"))), str!("id")))))
            ))
        })];

        let Filter { relations, .. } = Filter::build(types, ors, "resource", "Resource")?;
        assert_eq!(
            relations,
            vec![
                Relation(s("Resource"), s("foo"), s("Foo")),
                Relation(s("Foo"), s("boo"), s("Boo")),
                Relation(s("Boo"), s("goo"), s("Goo"))
            ]
        );

        Ok(())
    }
}
