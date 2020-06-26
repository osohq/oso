//! # Types
//!
//! Polar types

use serde::{Deserialize, Serialize};

use std::collections::{BTreeMap, HashMap};
use std::convert::TryFrom;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::{error, ToPolarString};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default)]
pub struct Dictionary {
    pub fields: BTreeMap<Symbol, Term>,
}

impl Dictionary {
    pub fn new() -> Self {
        Self {
            fields: BTreeMap::new(),
        }
    }

    fn map_replace<F>(&mut self, f: &mut F)
    where
        F: FnMut(&Term) -> Term,
    {
        self.fields.iter_mut().for_each(|(_k, v)| v.map_replace(f));
    }

    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    /// Convert all terms in this dictionary to patterns.
    pub fn as_pattern(&self) -> Pattern {
        let mut pattern = self.clone();
        pattern.map_replace(&mut |t| {
            let v = Pattern::value_as_pattern(t.value());
            t.clone_with_value(v)
        });
        Pattern::Dictionary(pattern)
    }
}

pub fn field_name(field: &Term) -> Symbol {
    if let Value::Call(Predicate { name, .. }) = &field.value() {
        name.clone()
    } else {
        panic!("keys must be symbols; received: {:?}", field.value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct InstanceLiteral {
    pub tag: Symbol,
    pub fields: Dictionary,
}

impl InstanceLiteral {
    pub fn map_replace<F>(&mut self, f: &mut F)
    where
        F: FnMut(&Term) -> Term,
    {
        self.fields
            .fields
            .iter_mut()
            .for_each(|(_, v)| v.map_replace(f));
    }

    /// Convert all terms in this instance literal to patterns.
    pub fn as_pattern(&self) -> Pattern {
        let mut pattern = self.clone();
        pattern.map_replace(&mut |t| {
            let v = Pattern::value_as_pattern(t.value());
            t.clone_with_value(v)
        });
        Pattern::Instance(pattern)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct ExternalInstance {
    pub instance_id: u64,
    pub literal: Option<InstanceLiteral>,
}

// Context stored somewhere by id.

// parser outputs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    pub file: String,
    // TODO: more things

    // maybe for ffi, you say the method on what python class you called or whatever.
}

pub type TermList = Vec<Term>;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Symbol(pub String);

impl Symbol {
    pub fn new(name: &str) -> Self {
        Self(name.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Predicate {
    pub name: Symbol,
    pub args: TermList,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum Operator {
    Debug,
    Cut,
    In,
    Isa,
    New,
    Dot,
    Not,
    Mul,
    Div,
    Add,
    Sub,
    Eq,
    Geq,
    Leq,
    Neq,
    Gt,
    Lt,
    Unify,
    Or,
    And,
    ForAll,
}

impl Operator {
    pub fn precedence(self) -> i32 {
        match self {
            Operator::Debug => 11,
            Operator::New => 10,
            Operator::Cut => 10,
            Operator::ForAll => 10,
            Operator::Dot => 9,
            Operator::In => 8,
            Operator::Isa => 8,
            Operator::Not => 7,
            Operator::Mul => 6,
            Operator::Div => 6,
            Operator::Add => 5,
            Operator::Sub => 5,
            Operator::Eq => 4,
            Operator::Geq => 4,
            Operator::Leq => 4,
            Operator::Neq => 4,
            Operator::Gt => 4,
            Operator::Lt => 4,
            Operator::Unify => 3,
            Operator::Or => 2,
            Operator::And => 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Operation {
    pub operator: Operator,
    pub args: TermList,
}

/// Represents a pattern in a specializer or after isa.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum Pattern {
    Dictionary(Dictionary),
    Instance(InstanceLiteral),
}

impl Pattern {
    pub fn value_as_pattern(value: &Value) -> Value {
        match value.clone() {
            Value::InstanceLiteral(lit) => Value::Pattern(lit.as_pattern()),
            Value::Dictionary(dict) => Value::Pattern(dict.as_pattern()),
            v => v,
        }
    }

    pub fn term_as_pattern(term: &Term) -> Term {
        term.clone_with_value(Self::value_as_pattern(term.value()))
    }
}

pub type Float = ordered_float::OrderedFloat<f64>;

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Eq)]
pub enum Numeric {
    Integer(i64),
    Float(Float),
}

impl PartialEq for Numeric {
    fn eq(&self, other: &Self) -> bool {
        matches!(self.partial_cmp(other), Some(std::cmp::Ordering::Equal))
    }
}

impl PartialOrd for Numeric {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // compare the integer `i` and the float `f`
        // if `swap` then do `f.partial_cmp(i)` otherwise do `i.partial_cmp(f)`
        let cmp_and_swap = |i: i64, f: Float, swap: bool| {
            if let Ok(i) = u32::try_from(i) {
                // integer and float are equal if they are within Ɛ of each other
                if (f.into_inner() - f64::from(i)).abs() <= f64::EPSILON {
                    Some(std::cmp::Ordering::Equal)
                } else if swap {
                    f.into_inner().partial_cmp(&f64::from(i))
                } else {
                    f64::from(i).partial_cmp(&f)
                }
            } else {
                None
            }
        };
        match (*self, *other) {
            (Self::Integer(left), Self::Integer(right)) => left.partial_cmp(&right),
            (Self::Integer(i), Self::Float(f)) => cmp_and_swap(i, f, false),
            (Self::Float(f), Self::Integer(i)) => cmp_and_swap(i, f, true),
            (Self::Float(left), Self::Float(right)) => left.partial_cmp(&right),
        }
    }
}

impl From<i64> for Numeric {
    fn from(other: i64) -> Self {
        Self::Integer(other)
    }
}
impl From<f64> for Numeric {
    fn from(other: f64) -> Self {
        Self::Float(other.into())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum Value {
    Number(Numeric),
    String(String),
    Boolean(bool),
    ExternalInstance(ExternalInstance),
    // TODO (dhatch) Remove this type so that it is no longer possible to even make an
    // instance literal value!
    InstanceLiteral(InstanceLiteral),
    Dictionary(Dictionary),
    Pattern(Pattern),
    Call(Predicate), // @TODO: Do we just want a type for this instead?
    List(TermList),
    Symbol(Symbol),
    Expression(Operation),
}

impl Value {
    pub fn symbol(self) -> Result<Symbol, error::RuntimeError> {
        match self {
            Value::Symbol(name) => Ok(name),
            _ => Err(error::RuntimeError::TypeError {
                msg: format!("Expected symbol, got: {}", self.to_polar()),
                loc: 0,
                context: None, // @TODO
            }),
        }
    }

    pub fn instance_literal(self) -> Result<InstanceLiteral, error::RuntimeError> {
        match self {
            Value::InstanceLiteral(literal) => Ok(literal),
            _ => Err(error::RuntimeError::TypeError {
                msg: format!("Expected instance literal, got: {}", self.to_polar()),
                loc: 0,
                context: None, // @TODO
            }),
        }
    }

    pub fn expression(self) -> Result<Operation, error::RuntimeError> {
        match self {
            Value::Expression(op) => Ok(op),
            _ => Err(error::RuntimeError::TypeError {
                msg: format!("Expected instance literal, got: {}", self.to_polar()),
                loc: 0,
                context: None, // @TODO
            }),
        }
    }

    pub fn call(self) -> Result<Predicate, error::RuntimeError> {
        match self {
            Value::Call(pred) => Ok(pred),
            _ => Err(error::RuntimeError::TypeError {
                msg: format!("Expected instance literal, got: {}", self.to_polar()),
                loc: 0,
                context: None, // @TODO
            }),
        }
    }
}

#[derive(Debug, Clone)]
enum SourceInfo {
    // From the parser
    Parser {
        /// Index into the source map stored in the knowledge base
        src_id: u64,

        /// Location of the term within the source map
        offset: usize,
    },

    /// Created as a temporary variable
    TemporaryVariable,

    /// From an FFI call
    Ffi,

    /// Created for a test
    Test,
}

impl SourceInfo {
    fn ffi() -> Self {
        Self::Ffi
    }
}

/// Represents a concrete instance of a Polar value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Term {
    /// Information about where the term was created from
    #[serde(skip, default = "SourceInfo::ffi")]
    source_info: SourceInfo,

    /// The actual underlying value
    value: Rc<Value>,
}

impl PartialEq for Term {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for Term {}

impl Term {
    /// Creates a new term for a temporary variable
    pub fn new_temporary(value: Value) -> Self {
        Self {
            source_info: SourceInfo::TemporaryVariable,
            value: Rc::new(value),
        }
    }

    /// Creates a new term from the parser
    pub fn new_from_parser(src_id: u64, offset: usize, value: Value) -> Self {
        Self {
            source_info: SourceInfo::Parser { src_id, offset },
            value: Rc::new(value),
        }
    }

    /// Creates a new term from a test value
    pub fn new_from_test(value: Value) -> Self {
        Self {
            source_info: SourceInfo::Test,
            value: Rc::new(value),
        }
    }

    /// Create a new Term, cloning the source info of `self`
    /// but with the new `value`
    pub fn clone_with_value(&self, value: Value) -> Self {
        Self {
            source_info: self.source_info.clone(),
            value: Rc::new(value),
        }
    }

    /// Replace the `value` of self
    pub fn replace_value(&mut self, value: Value) {
        self.value = Rc::new(value);
    }

    /// Convenience wrapper around map_replace that clones the
    /// term before running `map_replace`, to return the new value
    pub fn cloned_map_replace<F>(&self, f: &mut F) -> Self
    where
        F: FnMut(&Term) -> Term,
    {
        let mut term = self.clone();
        term.map_replace(f);
        term
    }

    /// Visits every term in the tree, replaces the node with the evaluation of `f` on the node
    /// and then recurses to the children
    ///
    /// Warning: this does _a lot_ of cloning.
    pub fn map_replace<F>(&mut self, f: &mut F)
    where
        F: FnMut(&Term) -> Term,
    {
        *self = f(self);
        let mut value = self.value().clone();
        match value {
            Value::Number(_) | Value::String(_) | Value::Boolean(_) | Value::Symbol(_) => {}
            Value::List(ref mut terms) => terms.iter_mut().for_each(|t| t.map_replace(f)),
            Value::Call(ref mut predicate) => {
                predicate.args.iter_mut().for_each(|a| a.map_replace(f))
            }
            Value::Expression(Operation { ref mut args, .. }) => {
                args.iter_mut().for_each(|term| term.map_replace(f))
            }
            Value::InstanceLiteral(InstanceLiteral { ref mut fields, .. }) => {
                fields.fields.iter_mut().for_each(|(_, v)| v.map_replace(f))
            }
            Value::ExternalInstance(_) => {}
            Value::Dictionary(Dictionary { ref mut fields }) => {
                fields.iter_mut().for_each(|(_, v)| v.map_replace(f))
            }
            Value::Pattern(Pattern::Dictionary(Dictionary { ref mut fields })) => {
                fields.iter_mut().for_each(|(_, v)| v.map_replace(f))
            }
            Value::Pattern(Pattern::Instance(InstanceLiteral { ref mut fields, .. })) => {
                fields.fields.iter_mut().for_each(|(_, v)| v.map_replace(f))
            }
        };
        self.replace_value(value);
    }

    pub fn offset(&self) -> usize {
        if let SourceInfo::Parser { offset, .. } = self.source_info {
            offset
        } else {
            0
        }
    }

    /// Get a reference to the underlying data of this term
    pub fn value(&self) -> &Value {
        &self.value
    }
}

pub fn unwrap_and(term: Term) -> TermList {
    match term.value() {
        Value::Expression(Operation {
            operator: Operator::And,
            args,
        }) => args.clone(),
        _ => vec![term.clone()],
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Parameter {
    pub parameter: Option<Term>,
    pub specializer: Option<Term>,
}

impl Parameter {
    pub fn map_replace<F>(&mut self, f: &mut F)
    where
        F: FnMut(&Term) -> Term,
    {
        self.parameter.iter_mut().for_each(|p| p.map_replace(f));
        self.specializer.iter_mut().for_each(|p| p.map_replace(f));
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Rule {
    pub name: Symbol,
    pub params: Vec<Parameter>,
    pub body: Term,
}

impl Rule {
    pub fn map_replace<F>(&mut self, f: &mut F)
    where
        F: FnMut(&Term) -> Term,
    {
        self.params.iter_mut().for_each(|p| p.map_replace(f));
        self.body.map_replace(f);
    }
}

pub type Rules = Vec<Rule>;

#[derive(Clone)]
pub struct GenericRule {
    pub name: Symbol,
    pub rules: Rules,
}

impl GenericRule {
    pub fn new(name: Symbol, rules: Rules) -> Self {
        GenericRule { name, rules }
    }
}

#[derive(Clone)]
pub enum Type {
    Class { name: Symbol },
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Source {
    pub filename: Option<String>,
    pub src: String,
}

pub struct Sources {
    // Pair of maps to go from Term ID -> Source ID -> Source.
    sources: HashMap<u64, Source>,
    // term_sources: HashMap<u64, u64>,
}

impl Default for Sources {
    fn default() -> Self {
        let mut sources = HashMap::new();
        sources.insert(
            0,
            Source {
                filename: None,
                src: "<Unknown>".to_string(),
            },
        );
        Self { sources }
    }
}

impl Sources {
    pub fn add_source(&mut self, source: Source, id: u64) {
        self.sources.insert(id, source);
    }

    pub fn get_source(&self, term: &Term) -> Option<Source> {
        if let SourceInfo::Parser { src_id, .. } = term.source_info {
            self.sources.get(&src_id).cloned()
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Node {
    Rule(Rule),
    Term(Term),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Trace {
    pub node: Node,
    pub polar_str: String,
    pub children: Vec<Rc<Trace>>,
}

#[derive(Default)]
pub struct KnowledgeBase {
    pub types: HashMap<Symbol, Type>,
    pub rules: HashMap<Symbol, GenericRule>,
    pub sources: Sources,
    // For symbols returned from gensym
    gensym_counter: AtomicU64,
    // For call IDs, instance IDs, symbols, etc.
    id_counter: AtomicU64,
    pub inline_queries: Vec<Term>,
}

impl KnowledgeBase {
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
            rules: HashMap::new(),
            sources: Sources::default(),
            id_counter: AtomicU64::new(1),
            gensym_counter: AtomicU64::new(1),
            inline_queries: vec![],
        }
    }

    /// Return a monotonically increasing integer ID.
    pub fn new_id(&self) -> u64 {
        self.id_counter.fetch_add(1, Ordering::SeqCst)
    }

    /// Generate a new symbol.
    pub fn gensym(&self, prefix: &str) -> Symbol {
        let next = self.gensym_counter.fetch_add(1, Ordering::SeqCst);
        if prefix.starts_with('_') {
            Symbol(format!("{}_{}", prefix, next))
        } else {
            Symbol(format!("_{}_{}", prefix, next))
        }
    }

    /// Add a generic rule to the knowledge base.
    #[cfg(test)]
    pub fn add_generic_rule(&mut self, rule: GenericRule) {
        self.rules.insert(rule.name.clone(), rule);
    }
}

pub type Bindings = HashMap<Symbol, Term>;

#[allow(clippy::large_enum_variant)]
#[must_use]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryEvent {
    None,
    Debug {
        message: String,
    },

    Done,

    MakeExternal {
        instance_id: u64,
        instance: InstanceLiteral,
    },

    ExternalCall {
        /// Persistent id across all requests for results from the same external call.
        call_id: u64,
        /// Id of the external instance to make this call on. None for functions or constructors.
        instance_id: u64,
        /// Field name to lookup or function name to call. A class name indicates a constructor
        /// should be called.
        attribute: Symbol,
        /// List of arguments to use if this is a method call.
        args: Vec<Term>,
    },

    /// Checks if the instance is an instance of (or subclass of) the class_tag.
    ExternalIsa {
        call_id: u64,
        instance_id: u64,
        class_tag: Symbol,
    },

    /// Checks if the instance is more specifically and instance/subclass of A than B.
    ExternalIsSubSpecializer {
        call_id: u64,
        instance_id: u64,
        left_class_tag: Symbol,
        right_class_tag: Symbol,
    },

    Result {
        bindings: Bindings,
        trace: Option<Rc<Trace>>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn serialize_test() {
        let pred = Predicate {
            name: Symbol("foo".to_owned()),
            args: vec![Term::new_from_test(value!(0))],
        };
        assert_eq!(
            serde_json::to_string(&pred).unwrap(),
            r#"{"name":"foo","args":[{"value":{"Number":{"Integer":0}}}]}"#
        );
        let event = QueryEvent::ExternalCall {
            call_id: 2,
            instance_id: 3,
            attribute: Symbol::new("foo"),
            args: vec![
                Term::new_from_test(value!(0)),
                Term::new_from_test(value!("hello")),
            ],
        };
        eprintln!("{}", serde_json::to_string(&event).unwrap());
        let term = Term::new_from_test(value!(1));
        eprintln!("{}", serde_json::to_string(&term).unwrap());
        let mut fields = BTreeMap::new();
        fields.insert(Symbol::new("hello"), term!(1234));
        fields.insert(
            Symbol::new("world"),
            Term::new_from_test(Value::String("something".to_owned())),
        );
        let literal = InstanceLiteral {
            tag: Symbol::new("Foo"),
            fields: Dictionary { fields },
        };
        let event = QueryEvent::MakeExternal {
            instance_id: 12345,
            instance: literal,
        };
        eprintln!("{}", serde_json::to_string(&event).unwrap());
        let external = Term::new_from_test(Value::ExternalInstance(ExternalInstance {
            instance_id: 12345,
            literal: None,
        }));
        let list_of = Term::new_from_test(Value::List(vec![external]));
        eprintln!("{}", serde_json::to_string(&list_of).unwrap());
        let mut fields = BTreeMap::new();
        fields.insert(Symbol::new("foo"), list_of);
        let dict = Term::new_from_test(Value::Dictionary(Dictionary { fields }));
        eprintln!("{}", serde_json::to_string(&dict).unwrap());
        let e = error::ParseError::InvalidTokenCharacter {
            token: "Integer".to_owned(),
            c: 'x',
            loc: 99,
            context: None,
        };
        let err: crate::PolarError = e.into();
        eprintln!("{}", serde_json::to_string(&err).unwrap());
        let rule = Rule {
            name: Symbol::new("foo"),
            params: vec![],
            body: Term::new_temporary(Value::Expression(Operation {
                operator: Operator::And,
                args: vec![dict.clone(), dict.clone(), dict.clone()],
            })),
        };
        eprintln!("{}", rule);
    }
}
