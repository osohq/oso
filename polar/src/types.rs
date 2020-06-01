//! # Types
//!
//! Polar types

use serde::{Deserialize, Serialize};

use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};

pub type SrcPos = (usize, usize);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParseError {
    IntegerOverflow { token: String, pos: SrcPos },
    InvalidTokenCharacter { token: String, c: char, pos: SrcPos }, //@TODO: Line and column instead of loc.
    InvalidToken { pos: SrcPos },
    UnrecognizedEOF { pos: SrcPos },
    UnrecognizedToken { token: String, pos: SrcPos },
    ExtraToken { token: String, pos: SrcPos },
}

// @TODO: Information about the context of the error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuntimeError {
    Serialization { msg: String },
    TypeError { msg: String },
    UnboundVariable { sym: Symbol },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationalError {
    Unimplemented(String),
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Parameter passed to function is invalid.
pub struct ParameterError(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolarError {
    Parse(ParseError),
    Runtime(RuntimeError),
    Operational(OperationalError),
    Parameter(ParameterError),
}

impl From<ParseError> for PolarError {
    fn from(err: ParseError) -> PolarError {
        PolarError::Parse(err)
    }
}

impl From<RuntimeError> for PolarError {
    fn from(err: RuntimeError) -> PolarError {
        PolarError::Runtime(err)
    }
}

impl From<OperationalError> for PolarError {
    fn from(err: OperationalError) -> PolarError {
        PolarError::Operational(err)
    }
}

impl From<ParameterError> for PolarError {
    fn from(err: ParameterError) -> PolarError {
        PolarError::Parameter(err)
    }
}

pub type PolarResult<T> = std::result::Result<T, PolarError>;

impl std::error::Error for PolarError {}

impl fmt::Display for PolarError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "todo")
        //match self {
        // PolarError::Parse(s) | PolarError::Serialization(s) | PolarError::Unimplemented(s) => {
        //     write!(f, "{}", s)
        // }
        // PolarError::Unknown => write!(f, "panic!"),
        //}
    }
}

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
    Make,
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
            Operator::Make => 9,
            Operator::Dot => 8,
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

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum Value {
    Integer(i64),
    String(String),
    Boolean(bool),
    ExternalInstance(ExternalInstance),
    InstanceLiteral(InstanceLiteral),
    Dictionary(Dictionary),
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
            Value::Integer(_) | Value::String(_) | Value::Boolean(_) | Value::Symbol(_) => {
                self.clone()
            }
            Value::List(terms) => Value::List(terms.iter().map(|term| term.map(f)).collect()),
            Value::Call(predicate) => Value::Call(predicate.map(f)),
            Value::Expression(Operation { operator, args }) => Value::Expression(Operation {
                operator: *operator,
                args: args.iter().map(|term| term.map(f)).collect(),
            }),
            Value::InstanceLiteral(InstanceLiteral { tag, fields }) => {
                Value::InstanceLiteral(InstanceLiteral {
                    tag: tag.clone(),
                    fields: fields.map(f),
                })
            }
            Value::ExternalInstance(_) => self.clone(),
            Value::Dictionary(dict) => Value::Dictionary(dict.map(f)),
        };
        // actually does the mapping of nodes: applies to all nodes, both leaves and
        // intermediate nodes
        f(&mapped)
    }

    pub fn symbol(self) -> Result<Symbol, Self> {
        match self {
            Value::Symbol(name) => Ok(name),
            _ => Err(self),
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

    // Generates ids for term and all sub terms. Returns vec of new term ids.
    pub fn gen_ids(&mut self, kb: &mut KnowledgeBase, new_ids: &mut Vec<u64>) {
        match &mut self.value {
            Value::Integer(_) => (),
            Value::String(_) => (),
            Value::Boolean(_) => (),
            Value::ExternalInstance(_) => (),
            Value::InstanceLiteral(instance) => {
                for (_, t) in &mut instance.fields.fields.iter_mut() {
                    t.gen_ids(kb, new_ids);
                }
            }
            Value::Dictionary(dict) => {
                for (_, t) in &mut dict.fields.iter_mut() {
                    t.gen_ids(kb, new_ids);
                }
            }
            Value::Call(pred) => {
                for t in &mut pred.args.iter_mut() {
                    t.gen_ids(kb, new_ids);
                }
            }
            Value::List(list) => {
                for t in &mut list.iter_mut() {
                    t.gen_ids(kb, new_ids);
                }
            }
            Value::Symbol(_) => (),
            Value::Expression(op) => {
                for t in &mut op.args.iter_mut() {
                    t.gen_ids(kb, new_ids);
                }
            }
        };
        if self.id == 0 {
            let new_id = kb.new_id();
            self.id = new_id;
            new_ids.push(new_id);
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
pub struct Class {
    pub name: Symbol,
}

#[derive(Clone)]
pub enum Type {
    Class { class: Class },
    Group { members: Vec<Type> },
}

pub enum Source {
    Query {
        src: Option<String>,
    },
    Load {
        filename: Option<String>,
        src: String,
    },
}

#[derive(Default)]
pub struct KnowledgeBase {
    pub types: HashMap<Symbol, Type>,
    pub rules: HashMap<Symbol, GenericRule>,

    pub sources: HashMap<u64, Source>,
    pub term_sources: HashMap<u64, u64>,

    // For temporary variable names, call IDs, instance IDs, symbols, etc.
    counter: AtomicU64,
}

impl KnowledgeBase {
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
            rules: HashMap::new(),
            sources: HashMap::new(),
            term_sources: HashMap::new(),
            counter: AtomicU64::new(1),
        }
    }

    /// Return a monotonically increasing integer ID.
    pub fn new_id(&self) -> u64 {
        self.counter.fetch_add(1, Ordering::SeqCst)
    }

    /// Generate a new symbol.
    pub fn gensym(&self, prefix: &str) -> Symbol {
        if prefix.starts_with('_') {
            Symbol(format!("{}_{}", prefix, self.new_id()))
        } else {
            Symbol(format!("_{}_{}", prefix, self.new_id()))
        }
    }

    /// Add a generic rule to the knowledge base.
    pub fn add_generic_rule(&mut self, rule: GenericRule) {
        self.rules.insert(rule.name.clone(), rule);
    }
}

type Bindings = HashMap<Symbol, Term>;

#[must_use]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryEvent {
    Done,

    MakeExternal {
        instance_id: u64,
        instance: InstanceLiteral,
    },

    ExternalCall {
        /// Persistent id across all requests for results from the same external call.
        call_id: u64,
        /// Id of the external instance to make this call on.
        instance_id: u64,
        /// Field name to lookup or function name to call.
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

    ExternalUnify {
        call_id: u64,
        left_instance_id: u64,
        right_instance_id: u64,
    },

    Result {
        bindings: Bindings,
    },
    Breakpoint,
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
                value: Value::Integer(0),
            }],
        };
        assert_eq!(
            serde_json::to_string(&pred).unwrap(),
            r#"{"name":"foo","args":[{"id":2,"offset":0,"value":{"Integer":0}}]}"#
        );
        let event = QueryEvent::ExternalCall {
            call_id: 2,
            instance_id: 3,
            attribute: Symbol::new("foo"),
            args: vec![
                Term {
                    id: 2,
                    offset: 0,
                    value: Value::Integer(0),
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
            value: Value::Integer(1),
        };
        eprintln!("{}", serde_json::to_string(&term).unwrap());
        let mut fields = BTreeMap::new();
        fields.insert(Symbol::new("hello"), Term::new(Value::Integer(1234)));
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
        let e = ParseError::InvalidTokenCharacter {
            token: "Integer".to_owned(),
            c: 'x',
            pos: (99, 99),
        };
        let er: PolarError = e.into();
        eprintln!("{}", serde_json::to_string(&er).unwrap());
    }
}
