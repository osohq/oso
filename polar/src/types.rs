//! # Types
//!
//! Polar types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// @TODO: Do some work to make these errors nice, really rough right now.
#[derive(Debug)]
pub enum PolarError {
    Parse(String),
    Serialization(String),
    Unimplemented(String),
    Unknown, // Type we return if we panic, the trace gets printed to stderr by default.
}

pub type PolarResult<T> = std::result::Result<T, PolarError>;

impl ToString for PolarError {
    fn to_string(&self) -> String {
        match self {
            PolarError::Parse(s) => s.to_string(),
            PolarError::Serialization(s) => s.to_string(),
            PolarError::Unimplemented(s) => s.to_string(),
            PolarError::Unknown => "panic!".to_string(),
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Dictionary {
    pub fields: HashMap<Symbol, Term>,
}

impl Dictionary {
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }
}

pub fn field_name(field: &Term) -> Symbol {
    if let Value::Call(Predicate { name, .. }) = &field.value {
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

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct ExternalInstance {
    pub instance_id: u64,
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

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
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

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Operation {
    pub operator: Operator,
    pub args: TermList,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum Value {
    Integer(i64),
    String(String),
    Boolean(bool),
    ExternalInstance(ExternalInstance),
    InstanceLiteral(InstanceLiteral), // Parsed, don't know what kind it is yet.
    ExternalInstanceLiteral(InstanceLiteral), // Used in rewrite for constructors.
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
        match self {
            Value::Integer(_) | Value::String(_) | Value::Boolean(_) | Value::Symbol(_) => f(&self),
            Value::List(terms) => Value::List(terms.iter().map(|term| term.map(f)).collect()),
            Value::Call(predicate) => Value::Call(predicate.map(f)),
            Value::Expression(Operation { operator, args }) => Value::Expression(Operation {
                operator: *operator,
                args: args.iter().map(|term| term.map(f)).collect(),
            }),
            Value::InstanceLiteral(_) => unimplemented!(),
            Value::ExternalInstance(_) => unimplemented!(),
            Value::ExternalInstanceLiteral(_) => unimplemented!(),
            Value::Dictionary(_) => unimplemented!(),
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

// Knowledge base internal types.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Rule {
    pub name: Symbol,
    pub params: TermList,
    pub body: Term,
}

#[derive(Clone)]
pub struct GenericRule {
    pub name: Symbol,
    pub rules: Vec<Rule>,
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

#[derive(Default, Clone)]
pub struct KnowledgeBase {
    pub types: HashMap<Symbol, Type>,
    pub rules: HashMap<Symbol, GenericRule>,

    // For temporary variable names, call IDs, instance IDs, symbols, etc.
    counter: usize,
}

impl KnowledgeBase {
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
            rules: HashMap::new(),
            counter: 1,
        }
    }

    /// Return a monotonically increasing integer ID.
    pub fn new_id(&mut self) -> u64 {
        let id = self.counter;
        self.counter += 1;
        id as u64
    }

    /// Generate a new symbol.
    pub fn gensym(&mut self, prefix: &str) -> Symbol {
        if prefix.starts_with('_') {
            Symbol(format!("{}_{}", prefix, self.new_id()))
        } else {
            Symbol(format!("_{}_{}", prefix, self.new_id()))
        }
    }
}

type Bindings = HashMap<Symbol, Term>;

#[must_use]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryEvent {
    Done,

    /// Returns: new instance id
    MakeExternal {
        instance_id: u64,
        instance: InstanceLiteral,
    },

    /// Returns: Term
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
    Result {
        bindings: Bindings,
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
        let mut fields = HashMap::new();
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
        }));
        let list_of = Term::new(Value::List(vec![external]));
        eprintln!("{}", serde_json::to_string(&list_of).unwrap());
        let mut fields = HashMap::new();
        fields.insert(Symbol::new("foo"), list_of);
        let dict = Term::new(Value::Dictionary(Dictionary { fields }));
        eprintln!("{}", serde_json::to_string(&dict).unwrap())
    }
}
