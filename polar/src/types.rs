use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::rc::Rc;

pub trait ToPolarString {
    fn to_polar(&self) -> String;
}

// AST type for polar expressions / rules / etc
// Internal knowledge base types.
// FFI types for passing polar values back and forth.
// FFI event types.
// Debugger events
//

// Knowledge base organized around rules.
// what's in those rules?

// @TODO flesh out.
// Internal only instance
// interal rep of external class (has fields, was constructed in polar)
// external only instance (id only)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct Instance {
    pub class: String,
    pub external_id: u64,
    //pub fields: HashMap<String, Term>,
}

impl ToPolarString for Instance {
    fn to_polar(&self) -> String {
        format!("Instance<{}>", self.class)
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
        format!("({})", self.iter().map(|t| t.to_polar()).collect::<Vec<String>>().join(","))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct Symbol(pub String);

impl ToPolarString for Symbol {
    fn to_polar(&self) -> String {
        format!("{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct Predicate {
    pub name: String,
    pub args: TermList,
}

impl ToPolarString for Predicate {
    fn to_polar(&self) -> String {
        format!("{}{}", self.name, self.args.to_polar())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum Value {
    Integer(i64),
    String(String),
    Boolean(bool),
    Instance(Instance),
    Call(Predicate), // @TODO: Do we just want a type for this instead?
    List(TermList),
    Symbol(Symbol),
}

impl ToPolarString for Value {
    fn to_polar(&self) -> String {
        match self {
            Value::Integer(i) => format!("{}", i),
            Value::String(s) => format!("\"{}\"", s),
            Value::Boolean(b) => {
                if *b {
                    format!("{}", "true")
                } else {
                    format!("{}", "false")
                }
            }
            Value::Instance(i) => i.to_polar(),
            Value::Call(c) => c.to_polar(),
            Value::List(l) => l.to_polar(),
            Value::Symbol(s) => s.to_polar(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct Term {
    pub id: u64,
    pub offset: usize,
    pub value: Value,
}

impl Term {
    pub fn new(value: Value) -> Self {
        Self { id: 0, offset: 0, value }
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

// internal knowledge base types.
#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct Rule {
    pub name: String,
    pub params: TermList,
    pub body: TermList,
}

impl ToPolarString for Rule {
    fn to_polar(&self) -> String {
        format!("{}{} := {};", self.name, self.params.to_polar(), self.body.to_polar())
    }
}

#[derive(Clone)]
pub struct GenericRule {
    pub name: String,
    pub rules: Vec<Rule>,
}

#[derive(Clone)]
pub struct Class {
    foo: i64,
}

#[derive(Clone)]
pub enum Type {
    Class { class: Class },
    // groups?
}

#[derive(Clone)]
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

pub type Env = Rc<Environment>;
pub type Bindings = HashMap<Symbol, Term>;

#[derive(Debug, Clone)]
pub struct Environment {
    bindings: Bindings,
    parent: Option<Rc<Environment>>,
}

// TODO: Might be able to shorten this a bit by having a special empty environment.

impl Environment {
    pub fn empty() -> Self {
        Environment {
            bindings: HashMap::new(),
            parent: None,
        }
    }

    pub fn new(parent: &Rc<Environment>) -> Self {
        Environment {
            bindings: HashMap::new(),
            parent: Some(Rc::clone(parent)),
        }
    }

    pub fn get(&self, symbol: &Symbol) -> Option<&Term> {
        if let Some(value) = self.bindings.get(symbol) {
            return Some(value);
        }

        if let Some(parent) = &self.parent {
            return parent.get(symbol);
        }

        None
    }

    pub fn set(&mut self, symbol: Symbol, value: Term) {
        self.bindings.insert(symbol, value);
    }

    pub fn contains(&self, symbol: &Symbol) -> bool {
        if self.bindings.contains_key(symbol) {
            return true;
        }

        if let Some(parent) = &self.parent {
            return parent.contains(symbol);
        }

        false
    }

    pub fn flatten_bindings(&self) -> Bindings {
        let mut bindings = self.bindings.clone();
        if let Some(parent) = &self.parent {
            let parent_bindings = parent.flatten_bindings();
            for (k, v) in parent_bindings.iter() {
                bindings.insert(k.clone(), v.clone());
            }
        }

        bindings
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryEvent {
    Done,
    External(Symbol), // POC
    Result { bindings: Bindings },
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
        println!("{}", serde_json::to_string(&pred).unwrap());
    }
}
