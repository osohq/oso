use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub trait ToPolarString {
    fn to_polar(&self) -> String;
}

// AST type for polar expressions / rules / etc.
// Internal knowledge base types.
// FFI types for passing polar values back and forth.
// FFI event types.
// Debugger events.
//

// @TODO flesh out.
// Internal only instance
// interal rep of external class (has fields, was constructed in polar)
// external only instance (id only)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Instance {
    pub class: String,
    pub external_id: u64,
    pub fields: HashMap<Symbol, Term>,
}

impl ToPolarString for Instance {
    fn to_polar(&self) -> String {
        let fields = self
            .fields
            .iter()
            .map(|(k, v)| format!("{}: {}", k.to_polar(), v.to_polar()))
            .collect::<Vec<String>>()
            .join(", ");
        format!("{}{{{}}}", self.class, fields)
    }
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

impl ToPolarString for TermList {
    fn to_polar(&self) -> String {
        format!(
            "({})",
            self.iter()
                .map(|t| t.to_polar())
                .collect::<Vec<String>>()
                .join(",")
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct Symbol(pub String);

impl ToPolarString for Symbol {
    fn to_polar(&self) -> String {
        format!("{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Predicate {
    pub name: String,
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

impl ToPolarString for Predicate {
    fn to_polar(&self) -> String {
        format!("{}{}", self.name, self.args.to_polar())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum Value {
    Integer(i64),
    String(String),
    Boolean(bool),
    Instance(Instance),
    Call(Predicate), // @TODO: Do we just want a type for this instead?
    List(TermList),
    Symbol(Symbol),
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
            Value::Instance(_) => unimplemented!(),
        }
    }
}

impl ToPolarString for Value {
    fn to_polar(&self) -> String {
        match self {
            Value::Integer(i) => format!("{}", i),
            Value::String(s) => format!("\"{}\"", s),
            Value::Boolean(b) => format!("{}", {
                if *b {
                    "true"
                } else {
                    "false"
                }
            }),
            Value::Instance(i) => i.to_polar(),
            Value::Call(c) => c.to_polar(),
            Value::List(l) => l.to_polar(),
            Value::Symbol(s) => s.to_polar(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Term {
    pub id: u64,
    pub offset: usize,
    pub value: Value,
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

impl ToPolarString for Term {
    fn to_polar(&self) -> String {
        self.value.to_polar()
    }
}

// steve here's how u parse stuff
// ( + 1 2 (* 3 4))
// => is(+(1, 1, *(3, 4))
// foo.bar
// => .(foo, bar, result)
// foo.bar(1,2 3)
// => .(foo, bar(1,2,3), result)
// foo(a) := baz(a);
// :=(foo(a), baz(a))
// foo(a: Foo{a: b}) := baz(a);
// :=(foo(), baz(a))

// Knowledge base internal types.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Rule {
    pub name: String,
    pub params: TermList,
    pub body: TermList,
}

impl ToPolarString for Rule {
    fn to_polar(&self) -> String {
        format!(
            "{}{} := {};",
            self.name,
            self.params.to_polar(),
            self.body.to_polar()
        )
    }
}

#[derive(Clone)]
pub struct GenericRule {
    pub name: String,
    pub rules: Vec<Rule>,
}

#[derive(Clone)]
pub struct Class {
    pub id: i64,
    pub name: String,
}

#[derive(Clone)]
pub enum Type {
    Class { class: Class },
    Group { members: Vec<Type> },
}

#[derive(Default, Clone)]
pub struct KnowledgeBase {
    pub types: HashMap<String, Type>,
    pub rules: HashMap<String, GenericRule>,
}

impl KnowledgeBase {
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
            rules: HashMap::new(),
        }
    }
}

type Bindings = HashMap<Symbol, Term>;

#[must_use]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryEvent {
    Done,
    ExternalConstructor {
        instance: Instance,
    },
    ExternalCall {
        call_id: i64,
        instance_id: i64,
        class: String,
        attribute: String,
        args: Vec<Term>,
    },
    TestExternal {
        name: Symbol, // POC
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
            name: "foo".to_owned(),
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
    }
}
