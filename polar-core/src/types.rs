//! # Types
//!
//! Polar types

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

pub use super::{error, formatting::ToPolarString};

/// A map of bindings: variable name → value. The VM uses a stack internally,
/// but can translate to and from this type.
pub type Bindings = HashMap<Symbol, Term>;

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
    pub constructor: Option<Term>,
    pub repr: Option<String>,
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

/// Return true if the list ends with a rest-variable.
#[allow(clippy::ptr_arg)]
pub fn has_rest_var(list: &TermList) -> bool {
    !list.is_empty() && matches!(list.last().unwrap().value(), Value::RestVariable(_))
}

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
    Print,
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
    Assign,
}

impl Operator {
    pub fn precedence(self) -> i32 {
        match self {
            Operator::Print => 11,
            Operator::Debug => 11,
            Operator::New => 10,
            Operator::Cut => 10,
            Operator::ForAll => 10,
            Operator::Dot => 9,
            Operator::In => 8,
            Operator::Isa => 8,
            Operator::Mul => 7,
            Operator::Div => 7,
            Operator::Add => 6,
            Operator::Sub => 6,
            Operator::Eq => 5,
            Operator::Geq => 5,
            Operator::Leq => 5,
            Operator::Neq => 5,
            Operator::Gt => 5,
            Operator::Lt => 5,
            Operator::Unify => 4,
            Operator::Assign => 4,
            Operator::Not => 3,
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

/// A number. See the [`numerics`] module for implementations.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Numeric {
    Integer(i64),
    Float(f64),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    Variable(Symbol),
    RestVariable(Symbol),
    Expression(Operation),
}

impl Value {
    pub fn symbol(self) -> Result<Symbol, error::RuntimeError> {
        match self {
            Value::Variable(name) => Ok(name),
            Value::RestVariable(name) => Ok(name),
            _ => Err(error::RuntimeError::TypeError {
                msg: format!("Expected symbol, got: {}", self.to_polar()),
                stack_trace: None, // @TODO
            }),
        }
    }

    pub fn instance_literal(self) -> Result<InstanceLiteral, error::RuntimeError> {
        match self {
            Value::InstanceLiteral(literal) => Ok(literal),
            _ => Err(error::RuntimeError::TypeError {
                msg: format!("Expected instance literal, got: {}", self.to_polar()),
                stack_trace: None, // @TODO
            }),
        }
    }

    pub fn expression(self) -> Result<Operation, error::RuntimeError> {
        match self {
            Value::Expression(op) => Ok(op),
            _ => Err(error::RuntimeError::TypeError {
                msg: format!("Expected instance literal, got: {}", self.to_polar()),
                stack_trace: None, // @TODO
            }),
        }
    }

    pub fn call(self) -> Result<Predicate, error::RuntimeError> {
        match self {
            Value::Call(pred) => Ok(pred),
            _ => Err(error::RuntimeError::TypeError {
                msg: format!("Expected instance literal, got: {}", self.to_polar()),
                stack_trace: None, // @TODO
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
        left: usize,
        right: usize,
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
    value: Arc<Value>,
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
            value: Arc::new(value),
        }
    }

    /// Creates a new term from the parser
    pub fn new_from_parser(src_id: u64, left: usize, right: usize, value: Value) -> Self {
        Self {
            source_info: SourceInfo::Parser {
                src_id,
                left,
                right,
            },
            value: Arc::new(value),
        }
    }

    /// Creates a new term from a test value
    pub fn new_from_test(value: Value) -> Self {
        Self {
            source_info: SourceInfo::Test,
            value: Arc::new(value),
        }
    }

    /// Create a new Term, cloning the source info of `self`
    /// but with the new `value`
    pub fn clone_with_value(&self, value: Value) -> Self {
        Self {
            source_info: self.source_info.clone(),
            value: Arc::new(value),
        }
    }

    /// Replace the `value` of self
    pub fn replace_value(&mut self, value: Value) {
        self.value = Arc::new(value);
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
            Value::Number(_)
            | Value::String(_)
            | Value::Boolean(_)
            | Value::Variable(_)
            | Value::RestVariable(_) => {}
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
        if let SourceInfo::Parser { left, .. } = self.source_info {
            left
        } else {
            0
        }
    }

    pub fn span(&self) -> Option<(usize, usize)> {
        if let SourceInfo::Parser { left, right, .. } = self.source_info {
            Some((left, right))
        } else {
            None
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
    pub parameter: Term,
    pub specializer: Option<Term>,
}

impl Parameter {
    pub fn map_replace<F>(&mut self, f: &mut F)
    where
        F: FnMut(&Term) -> Term,
    {
        self.parameter.map_replace(f);
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

pub type Rules = Vec<Arc<Rule>>;

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
    Rule(Arc<Rule>),
    Term(Term),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Trace {
    pub node: Node,
    pub children: Vec<Rc<Trace>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TraceResult {
    pub trace: Rc<Trace>,
    pub formatted: String,
}

#[derive(Default)]
pub struct KnowledgeBase {
    pub constants: Bindings,
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
            constants: HashMap::new(),
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
        if prefix == "_" {
            Symbol(format!("_{}", next))
        } else if prefix.starts_with('_') {
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

    /// Define a constant variable.
    pub fn constant(&mut self, name: Symbol, value: Term) {
        self.constants.insert(name, value);
    }

    /// Return true if a constant with the given name has been defined.
    pub fn is_constant(&self, name: &Symbol) -> bool {
        self.constants.contains_key(name)
    }
}

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
        constructor: Term,
    },

    ExternalCall {
        /// Persistent id across all requests for results from the same external call.
        call_id: u64,
        /// The external instance to make this call on. None for functions or constructors.
        instance: Option<Term>,
        /// Field name to lookup or method name to call. A class name indicates a constructor
        /// should be called.
        attribute: Symbol,
        /// List of arguments to use if this is a method call.
        args: Vec<Term>,
    },

    /// Checks if the instance is an instance of (a subclass of) the class_tag.
    ExternalIsa {
        call_id: u64,
        instance: Term,
        class_tag: Symbol,
    },

    /// Checks if the instance is more specifically and instance/subclass of A than B.
    ExternalIsSubSpecializer {
        call_id: u64,
        instance_id: u64,
        left_class_tag: Symbol,
        right_class_tag: Symbol,
    },

    /// Unifies two external instances.
    ExternalUnify {
        call_id: u64,
        left_instance_id: u64,
        right_instance_id: u64,
    },

    Result {
        bindings: Bindings,
        trace: Option<TraceResult>,
    },

    ExternalOp {
        call_id: u64,
        operator: Operator,
        args: TermList,
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
            instance: None,
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
            constructor: Term::new_from_test(Value::InstanceLiteral(literal)),
        };
        eprintln!("{}", serde_json::to_string(&event).unwrap());
        let external = Term::new_from_test(Value::ExternalInstance(ExternalInstance {
            instance_id: 12345,
            constructor: None,
            repr: None,
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
        };
        let err: crate::error::PolarError = e.into();
        eprintln!("{}", serde_json::to_string(&err).unwrap());
        let rule = Rule {
            name: Symbol::new("foo"),
            params: vec![],
            body: Term::new_temporary(Value::Expression(Operation {
                operator: Operator::And,
                args: vec![dict.clone(), dict.clone(), dict],
            })),
        };
        eprintln!("{}", rule);
    }
}
