//! # Types
//!
//! Polar types

use serde::{Deserialize, Serialize};

use std::collections::{BTreeMap, HashMap};
use std::convert::TryFrom;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::{error, ToPolarString};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash, Default)]
pub struct Dictionary {
    pub fields: BTreeMap<Symbol, Term>,
}

impl Dictionary {
    pub fn new() -> Self {
        Self {
            fields: BTreeMap::new(),
        }
    }

    fn map<F>(&self, f: &mut F) -> Dictionary
    where
        F: FnMut(&Value) -> Value,
    {
        Dictionary {
            fields: self
                .fields
                .iter()
                .map(|(k, v)| (k.clone(), v.map(f)))
                .collect(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    /// Convert all terms in this dictionary to patterns.
    pub fn as_pattern(&self) -> Pattern {
        Pattern::Dictionary(self.map(&mut Pattern::value_as_pattern))
    }
}

pub fn field_name(field: &Term) -> Symbol {
    if let Value::Call(Predicate { name, .. }) = &field.value {
        name.clone()
    } else {
        panic!("keys must be symbols; received: {:?}", field.value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct InstanceLiteral {
    pub tag: Symbol,
    pub fields: Dictionary,
}

impl InstanceLiteral {
    pub fn map<F>(&self, f: &mut F) -> InstanceLiteral
    where
        F: FnMut(&Value) -> Value,
    {
        InstanceLiteral {
            tag: self.tag.clone(),
            fields: self.fields.map(f),
        }
    }

    pub fn walk_mut<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut Term) -> bool,
    {
        self.fields
            .fields
            .iter_mut()
            .for_each(|(_, v)| v.walk_mut(f));
    }

    /// Convert all terms in this instance literal to patterns.
    pub fn as_pattern(&self) -> Pattern {
        Pattern::Instance(self.map(&mut Pattern::value_as_pattern))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Symbol(pub String);

impl Symbol {
    pub fn new(name: &str) -> Self {
        Self(name.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct Predicate {
    pub name: Symbol,
    pub args: TermList,
}

impl Predicate {
    fn map<F>(&self, f: &mut F) -> Predicate
    where
        F: FnMut(&Value) -> Value,
    {
        Predicate {
            name: self.name.clone(),
            args: self.args.iter().map(|term| term.map(f)).collect(),
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
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
}

impl Operator {
    pub fn precedence(self) -> i32 {
        match self {
            Operator::Debug => 11,
            Operator::New => 10,
            Operator::Cut => 10,
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

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct Operation {
    pub operator: Operator,
    pub args: TermList,
}

/// Represents a pattern in a specializer or after isa.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum Pattern {
    Dictionary(Dictionary),
    Instance(InstanceLiteral),
}

impl Pattern {
    pub fn value_as_pattern(value: &Value) -> Value {
        value.map(&mut |v| match v {
            Value::InstanceLiteral(lit) => Value::Pattern(lit.as_pattern()),
            Value::Dictionary(dict) => Value::Pattern(dict.as_pattern()),
            _ => v.clone(),
        })
    }

    pub fn term_as_pattern(term: &Term) -> Term {
        term.map(&mut Pattern::value_as_pattern)
    }

    pub fn map<F>(&self, f: &mut F) -> Pattern
    where
        F: FnMut(&Value) -> Value,
    {
        match self {
            Pattern::Instance(lit) => Pattern::Instance(lit.map(f)),
            Pattern::Dictionary(dict) => Pattern::Dictionary(dict.map(f)),
        }
    }
}

pub type Float = ordered_float::OrderedFloat<f64>;

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Eq, Hash)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum Value {
    Number(Numeric),
    String(String),
    Boolean(bool),
    ExternalInstance(ExternalInstance),
    InstanceLiteral(InstanceLiteral),
    Dictionary(Dictionary),
    Pattern(Pattern),
    Call(Predicate), // @TODO: Do we just want a type for this instead?
    List(TermList),
    Symbol(Symbol),
    Expression(Operation),
}

impl Value {
    pub fn map<F>(&self, f: &mut F) -> Value
    where
        F: FnMut(&Value) -> Value,
    {
        // the match does the recursive calling of map
        let mapped = match self {
            Value::Number(_) | Value::String(_) | Value::Boolean(_) | Value::Symbol(_) => {
                self.clone()
            }
            Value::List(terms) => Value::List(terms.iter().map(|term| term.map(f)).collect()),
            Value::Call(predicate) => Value::Call(predicate.map(f)),
            Value::Expression(Operation { operator, args }) => Value::Expression(Operation {
                operator: *operator,
                args: args.iter().map(|term| term.map(f)).collect(),
            }),
            Value::InstanceLiteral(literal) => Value::InstanceLiteral(literal.map(f)),
            Value::ExternalInstance(_) => self.clone(),
            Value::Dictionary(dict) => Value::Dictionary(dict.map(f)),
            Value::Pattern(pat) => Value::Pattern(pat.map(f)),
        };
        // actually does the mapping of nodes: applies to all nodes, both leaves and
        // intermediate nodes
        f(&mapped)
    }

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

#[derive(Debug, Clone, Serialize, Deserialize, Eq)]
pub struct Term {
    pub id: u64,
    pub offset: usize,
    pub value: Value,
}

impl PartialEq for Term {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Hash for Term {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl Term {
    pub fn new(value: Value) -> Self {
        Self {
            id: 0,
            offset: 0,
            value,
        }
    }

    pub fn clone_with_value(&self, value: Value) -> Self {
        Self {
            id: self.id,
            offset: self.offset,
            value,
        }
    }

    pub fn replace_value(&mut self, value: Value) -> Value {
        std::mem::replace(&mut self.value, value)
    }

    /// Apply `f` to value and return a new term.
    pub fn map<F>(&self, f: &mut F) -> Term
    where
        F: FnMut(&Value) -> Value,
    {
        Term {
            id: self.id,
            offset: self.offset,
            value: self.value.map(f),
        }
    }

    /// Does a preorder walk of the term tree, calling F on itself and then walking its children.
    /// If F returns true walk the children, otherwise stop.
    pub fn walk_mut<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut Self) -> bool,
    {
        let walk_children = f(self);
        if walk_children {
            match self.value {
                Value::Number(_) | Value::String(_) | Value::Boolean(_) | Value::Symbol(_) => {}
                Value::List(ref mut terms) => terms.iter_mut().for_each(|t| t.walk_mut(f)),
                Value::Call(ref mut predicate) => {
                    predicate.args.iter_mut().for_each(|a| a.walk_mut(f))
                }
                Value::Expression(Operation { ref mut args, .. }) => {
                    args.iter_mut().for_each(|term| term.walk_mut(f))
                }
                Value::InstanceLiteral(InstanceLiteral { ref mut fields, .. }) => {
                    fields.fields.iter_mut().for_each(|(_, v)| v.walk_mut(f))
                }
                Value::ExternalInstance(_) => {}
                Value::Dictionary(Dictionary { ref mut fields }) => {
                    fields.iter_mut().for_each(|(_, v)| v.walk_mut(f))
                }
                Value::Pattern(Pattern::Dictionary(Dictionary { ref mut fields })) => {
                    fields.iter_mut().for_each(|(_, v)| v.walk_mut(f))
                }
                Value::Pattern(Pattern::Instance(InstanceLiteral { ref mut fields, .. })) => {
                    fields.fields.iter_mut().for_each(|(_, v)| v.walk_mut(f))
                }
            };
        }
    }
}

pub fn unwrap_and(term: Term) -> TermList {
    match term.value {
        Value::Expression(Operation {
            operator: Operator::And,
            args,
        }) => args,
        _ => vec![term],
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Parameter {
    pub name: Option<Symbol>,
    pub specializer: Option<Term>,
}

impl Parameter {
    pub fn map<F>(&self, f: &mut F) -> Parameter
    where
        F: FnMut(&Value) -> Value,
    {
        let name = if let Some(name) = &self.name {
            if let Value::Symbol(new_sym) = f(&Value::Symbol(name.clone())) {
                Some(new_sym)
            } else {
                None
            }
        } else {
            None
        };

        Parameter {
            name,
            specializer: self.specializer.clone().map(|t| t.map(f)),
        }
    }

    /// Does a preorder walk of the parameter terms.
    pub fn walk_mut<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut Term) -> bool,
    {
        self.specializer.iter_mut().for_each(|mut a| {
            f(&mut a);
        });
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Rule {
    pub name: Symbol,
    pub params: Vec<Parameter>,
    pub body: Term,
}

impl Rule {
    pub fn map<F>(&self, f: &mut F) -> Rule
    where
        F: FnMut(&Value) -> Value,
    {
        Rule {
            name: self.name.clone(),
            params: self.params.iter().map(|param| param.map(f)).collect(),
            body: self.body.map(f),
        }
    }

    /// Does a preorder walk of the rule parameters and body.
    pub fn walk_mut<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut Term) -> bool,
    {
        self.params.iter_mut().for_each(|param| param.walk_mut(f));
        self.body.walk_mut(f);
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

#[derive(Default)]
pub struct Sources {
    // Pair of maps to go from Term ID -> Source ID -> Source.
    sources: HashMap<u64, Source>,
    term_sources: HashMap<u64, u64>,
}

impl Sources {
    pub fn add_source(&mut self, source: Source, id: u64) {
        self.sources.insert(id, source);
    }

    pub fn add_term_source(&mut self, term: &Term, src_id: u64) {
        self.term_sources.insert(term.id, src_id);
    }

    pub fn get_source(&self, term: &Term) -> Option<Source> {
        self.term_sources
            .get(&term.id)
            .and_then(|term_source| self.sources.get(&term_source).cloned())
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
    pub children: Vec<Trace>,
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
        trace: Option<Trace>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn serialize_test() {
        let pred = Predicate {
            name: Symbol("foo".to_owned()),
            args: vec![Term {
                id: 2,
                offset: 0,
                value: value!(0),
            }],
        };
        assert_eq!(
            serde_json::to_string(&pred).unwrap(),
            r#"{"name":"foo","args":[{"id":2,"offset":0,"value":{"Number":{"Integer":0}}}]}"#
        );
        let event = QueryEvent::ExternalCall {
            call_id: 2,
            instance_id: 3,
            attribute: Symbol::new("foo"),
            args: vec![
                Term {
                    id: 2,
                    offset: 0,
                    value: value!(0),
                },
                Term {
                    id: 3,
                    offset: 0,
                    value: Value::String("hello".to_string()),
                },
            ],
        };
        eprintln!("{}", serde_json::to_string(&event).unwrap());
        let term = Term {
            id: 0,
            offset: 0,
            value: value!(1),
        };
        eprintln!("{}", serde_json::to_string(&term).unwrap());
        let mut fields = BTreeMap::new();
        fields.insert(Symbol::new("hello"), term!(1234));
        fields.insert(
            Symbol::new("world"),
            Term::new(Value::String("something".to_owned())),
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
        let external = Term::new(Value::ExternalInstance(ExternalInstance {
            instance_id: 12345,
            literal: None,
        }));
        let list_of = Term::new(Value::List(vec![external]));
        eprintln!("{}", serde_json::to_string(&list_of).unwrap());
        let mut fields = BTreeMap::new();
        fields.insert(Symbol::new("foo"), list_of);
        let dict = Term::new(Value::Dictionary(Dictionary { fields }));
        eprintln!("{}", serde_json::to_string(&dict).unwrap());
        let e = error::ParseError::InvalidTokenCharacter {
            token: "Integer".to_owned(),
            c: 'x',
            loc: 99,
            context: None,
        };
        let err: crate::PolarError = e.into();
        eprintln!("{}", serde_json::to_string(&err).unwrap());
    }
}
