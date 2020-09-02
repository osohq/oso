use super::sources::SourceInfo;
pub use super::{error, formatting::ToPolarString};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use super::numerics::Numeric;

#[derive(Debug, Clone, Serialize, Deserialize, Default, Eq, PartialEq, Hash)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct Predicate {
    pub name: Symbol,
    pub args: TermList,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Value {
    Number(Numeric),
    String(String),
    Boolean(bool),
    ExternalInstance(ExternalInstance),
    InstanceLiteral(InstanceLiteral),
    Dictionary(Dictionary),
    Pattern(Pattern),
    Call(Predicate),
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

    pub fn is_ground(&self) -> bool {
        match self {
            Value::Call(_)
            | Value::ExternalInstance(_)
            | Value::Variable(_)
            | Value::RestVariable(_) => false,
            Value::Number(_) | Value::String(_) | Value::Boolean(_) => true,
            Value::InstanceLiteral(_) | Value::Pattern(_) => panic!("unexpected value type"),
            Value::Dictionary(Dictionary { fields }) => fields.values().all(|t| t.is_ground()),
            Value::List(terms) => terms.iter().all(|t| t.is_ground()),
            Value::Expression(Operation { operator: _, args }) => {
                args.iter().all(|t| t.is_ground())
            }
        }
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

impl Hash for Term {
    /// Hash just the value, not source information.
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.value().hash(state)
    }
}

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
            Value::ExternalInstance(ExternalInstance {
                ref mut constructor,
                ..
            }) => constructor.iter_mut().for_each(|t| t.map_replace(f)),
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

    pub fn is_ground(&self) -> bool {
        self.value().is_ground()
    }

    /// Get a set of all the variables used within a term.
    pub fn variables(&self, vars: &mut HashSet<Symbol>) {
        self.cloned_map_replace(&mut |term| {
            if let Value::Variable(s) = term.value() {
                vars.insert(s.clone());
            }
            term.clone()
        });
    }

    pub fn get_source_id(&self) -> Option<u64> {
        if let SourceInfo::Parser { src_id, .. } = self.source_info {
            Some(src_id)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_value_hash() {
        let mut table = HashMap::new();
        table.insert(value!(0), "0");
        table.insert(value!(1), "1");
        table.insert(value!("one"), "one");
        table.insert(value!(btreemap! {sym!("a") => term!(1)}), "a:1");
        table.insert(value!(btreemap! {sym!("b") => term!(2)}), "b:2");
        assert_eq!(*table.get(&value!(0)).unwrap(), "0");
        assert_eq!(*table.get(&value!(1)).unwrap(), "1");
        assert_eq!(*table.get(&value!(1.0)).unwrap(), "1");
        assert_eq!(*table.get(&value!("one")).unwrap(), "one");
        assert_eq!(
            *table
                .get(&value!(btreemap! {sym!("a") => term!(1)}))
                .unwrap(),
            "a:1"
        );
        assert_eq!(
            *table
                .get(&value!(btreemap! {sym!("b") => term!(2)}))
                .unwrap(),
            "b:2"
        );
    }
}
