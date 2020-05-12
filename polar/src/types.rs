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

pub trait ToPolarString {
    fn to_polar(&self) -> String;
}

/// PARSER TYPES (NOT FFI TYPE BECAUSE THIS can't cross boundary)
/// Not necessarily true due to instance contruction.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct InstanceLiteral {
    pub tag: String,
    pub fields: HashMap<Symbol, Term>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Dictionary {
    pub fields: HashMap<Symbol, Term>,
}

impl ToPolarString for Dictionary {
    fn to_polar(&self) -> String {
        unimplemented!();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct ExternalInstance {
    pub external_id: u64,
}

impl ToPolarString for ExternalInstance {
    fn to_polar(&self) -> String {
        unimplemented!();
    }
}

impl ToPolarString for InstanceLiteral {
    fn to_polar(&self) -> String {
        let fields = self
            .fields
            .iter()
            .map(|(k, v)| format!("{}: {}", k.to_polar(), v.to_polar()))
            .collect::<Vec<String>>()
            .join(", ");
        format!("{}{{{}}}", self.tag, fields)
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
        if self.args.is_empty() {
            format!("{}", self.name)
        } else {
            format!(
                "{}({})",
                self.name,
                self.args
                    .iter()
                    .map(|t| t.to_polar())
                    .collect::<Vec<String>>()
                    .join(",")
            )
        }
    }
}
#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum Operator {
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

pub fn op_precedence(op: &Operator) -> i32 {
    match op {
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

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Operation {
    pub operator: Operator,
    pub args: TermList,
}

fn has_lower_pred(op: &Operator, t: &Term) -> bool {
    match t.value {
        Value::Expression(Operation {
            operator: ref other,
            ..
        }) => op_precedence(op) > op_precedence(other),
        _ => false,
    }
}

fn to_polar_parens(op: &Operator, t: &Term) -> String {
    if has_lower_pred(op, t) {
        format!("({})", t.to_polar())
    } else {
        t.to_polar()
    }
}

// Adds parenthesis when sub expressions have lower precidence (which is what you would have had to have during inital parse)
// Lets us spit out strings that would reparse to the same ast.
impl ToPolarString for Operation {
    fn to_polar(&self) -> String {
        match self.operator {
            Operator::Dot => {
                if self.args.len() == 2 {
                    format!("{}.{}", self.args[0].to_polar(), self.args[1].to_polar())
                } else {
                    format!(
                        ".({})",
                        self.args
                            .iter()
                            .map(|t| to_polar_parens(&self.operator, t))
                            .collect::<Vec<String>>()
                            .join(","),
                    )
                }
            }
            Operator::Not => format!("{}{}", "!", self.args[0].to_polar()),
            Operator::Mul => format!(
                "{}*{}",
                to_polar_parens(&self.operator, &self.args[0]),
                to_polar_parens(&self.operator, &self.args[1])
            ),
            Operator::Div => format!(
                "{}/{}",
                to_polar_parens(&self.operator, &self.args[0]),
                to_polar_parens(&self.operator, &self.args[1])
            ),
            Operator::Add => format!(
                "{}+{}",
                to_polar_parens(&self.operator, &self.args[0]),
                to_polar_parens(&self.operator, &self.args[1])
            ),
            Operator::Sub => format!(
                "{}-{}",
                to_polar_parens(&self.operator, &self.args[0]),
                to_polar_parens(&self.operator, &self.args[1])
            ),
            Operator::Eq => format!(
                "{}=={}",
                to_polar_parens(&self.operator, &self.args[0]),
                to_polar_parens(&self.operator, &self.args[1])
            ),
            Operator::Geq => format!(
                "{}>={}",
                to_polar_parens(&self.operator, &self.args[0]),
                to_polar_parens(&self.operator, &self.args[1])
            ),
            Operator::Leq => format!(
                "{}<={}",
                to_polar_parens(&self.operator, &self.args[0]),
                to_polar_parens(&self.operator, &self.args[1])
            ),
            Operator::Neq => format!(
                "{}!={}",
                to_polar_parens(&self.operator, &self.args[0]),
                to_polar_parens(&self.operator, &self.args[1])
            ),
            Operator::Gt => format!(
                "{}>{}",
                to_polar_parens(&self.operator, &self.args[0]),
                to_polar_parens(&self.operator, &self.args[1]),
            ),
            Operator::Lt => format!(
                "{}<{}",
                to_polar_parens(&self.operator, &self.args[0]),
                to_polar_parens(&self.operator, &self.args[1]),
            ),
            Operator::Unify => format!(
                "{}={}",
                to_polar_parens(&self.operator, &self.args[0]),
                to_polar_parens(&self.operator, &self.args[1]),
            ),
            Operator::Or => self
                .args
                .iter()
                .map(|t| to_polar_parens(&self.operator, t))
                .collect::<Vec<String>>()
                .join("|"),
            Operator::And => self
                .args
                .iter()
                .map(|t| to_polar_parens(&self.operator, t))
                .collect::<Vec<String>>()
                .join(","),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
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
            Value::Dictionary(_) => unimplemented!(),
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
            Value::InstanceLiteral(i) => i.to_polar(),
            Value::Dictionary(i) => i.to_polar(),
            Value::ExternalInstance(i) => i.to_polar(),
            Value::Call(c) => c.to_polar(),
            Value::List(l) => format!(
                "[{}]",
                l.iter()
                    .map(|t| t.to_polar())
                    .collect::<Vec<String>>()
                    .join(",")
            ),
            Value::Symbol(s) => s.to_polar(),
            Value::Expression(e) => e.to_polar(),
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

impl ToPolarString for Term {
    fn to_polar(&self) -> String {
        self.value.to_polar()
    }
}

// Knowledge base internal types.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Rule {
    pub name: String,
    pub params: TermList,
    pub body: Term,
}

impl ToPolarString for Rule {
    fn to_polar(&self) -> String {
        match &self.body {
            Term {
                value:
                    Value::Expression(Operation {
                        operator: Operator::And,
                        args,
                    }),
                ..
            } => {
                if args.len() == 0 {
                    format!(
                        "{}({});",
                        self.name,
                        self.params
                            .iter()
                            .map(|t| t.to_polar())
                            .collect::<Vec<String>>()
                            .join(","),
                    )
                } else {
                    format!(
                        "{}({}) := {};",
                        self.name,
                        self.params
                            .iter()
                            .map(|t| t.to_polar())
                            .collect::<Vec<String>>()
                            .join(","),
                        args.iter()
                            .map(|t| t.to_polar())
                            .collect::<Vec<String>>()
                            .join(","),
                    )
                }
            }
            _ => panic!("Not any sorta rule I parsed"),
        }
    }
}

#[derive(Clone)]
pub struct GenericRule {
    pub name: String,
    pub rules: Vec<Rule>,
}

#[derive(Clone)]
pub struct Class {
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

    /// Returns: new instance id
    ExternalConstructor {
        instance: InstanceLiteral,
    },

    /// Returns: Term
    ExternalCall {
        /// Persistent id across all requests for results from the same external call.
        call_id: u64,
        /// Id of the external instance to make this call on.
        instance_id: u64,
        /// Field name to lookup or function name to call.
        attribute: String,
        /// List of arguments to use if this is a method call.
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
