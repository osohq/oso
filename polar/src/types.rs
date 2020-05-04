use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

// Context stored somewhere by id.

// parser outputs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    pub file: String,
    // TODO: more things

    // maybe for ffi, you say the method on what python class you called or whatever.
}

type TermList = Vec<Term>;

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct Symbol(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct Predicate {
    pub name: String,
    pub args: TermList
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum Value {
    Integer(i64),
    String(String),
    Boolean(bool),
    Instance(Instance),
    Call(Predicate),
    List(TermList),
    Symbol(Symbol),
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct Term {
    pub id: u64,
    pub value: Value
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
pub struct Rule {
    pub params: TermList,
    pub body: TermList,
}

pub struct GenericRule {
    pub name: String,
    pub rules: Vec<Rule>,
}

pub struct Class {
    foo: i64
}

pub enum Type {
    Class {class: Class},
    // groups?
}

pub struct KnowledgeBase {
    pub types: HashMap<String, Type>,
    pub rules: HashMap<String, GenericRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub bindings: HashMap<Symbol, Term>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryEvent {
    Done,
    Result {
        environment: Environment
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn serialize_test() {
        let pred = Predicate{ name: "foo".to_owned(), args: vec![Term{id: 2, value: Value::Integer(0)}]};
        println!("{}", serde_json::to_string(&pred).unwrap());
    }
}