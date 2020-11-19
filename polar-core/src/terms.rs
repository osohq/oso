use super::sources::SourceInfo;
pub use super::{error, formatting::ToPolarString};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

pub use super::numerics::Numeric;
use super::partial::Partial;
use super::visitor::{walk_term, Visitor};

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

    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
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

    pub fn is_temporary_var(&self) -> bool {
        self.0.starts_with('_')
    }

    pub fn is_namespaced_var(&self) -> bool {
        self.0.find("::").is_some()
    }

    pub fn is_this_var(&self) -> bool {
        self.0 == "_this"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct Call {
    pub name: Symbol,
    pub args: TermList,
    pub kwargs: Option<BTreeMap<Symbol, Term>>,
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
    Mod,
    Rem,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Value {
    Number(Numeric),
    String(String),
    Boolean(bool),
    ExternalInstance(ExternalInstance),
    InstanceLiteral(InstanceLiteral),
    Dictionary(Dictionary),
    Pattern(Pattern),
    Call(Call),
    List(TermList),
    Variable(Symbol),
    RestVariable(Symbol),
    Expression(Operation),
    Partial(Partial),
}

impl Value {
    pub fn as_symbol(&self) -> Result<&Symbol, error::RuntimeError> {
        match self {
            Value::Variable(name) => Ok(name),
            Value::RestVariable(name) => Ok(name),
            _ => Err(error::RuntimeError::TypeError {
                msg: format!("Expected symbol, got: {}", self.to_polar()),
                stack_trace: None, // @TODO
            }),
        }
    }

    pub fn as_string(&self) -> Result<&str, error::RuntimeError> {
        match self {
            Value::String(string) => Ok(string.as_ref()),
            _ => Err(error::RuntimeError::TypeError {
                msg: format!("Expected string, got: {}", self.to_polar()),
                stack_trace: None, // @TODO
            }),
        }
    }

    pub fn as_instance_literal(&self) -> Result<&InstanceLiteral, error::RuntimeError> {
        match self {
            Value::InstanceLiteral(literal) => Ok(literal),
            _ => Err(error::RuntimeError::TypeError {
                msg: format!("Expected instance literal, got: {}", self.to_polar()),
                stack_trace: None, // @TODO
            }),
        }
    }

    pub fn as_expression(&self) -> Result<&Operation, error::RuntimeError> {
        match self {
            Value::Expression(op) => Ok(op),
            _ => Err(error::RuntimeError::TypeError {
                msg: format!("Expected expression, got: {}", self.to_polar()),
                stack_trace: None, // @TODO
            }),
        }
    }

    pub fn as_partial(&self) -> Result<&Partial, error::RuntimeError> {
        match self {
            Value::Partial(e) => Ok(e),
            _ => Err(error::RuntimeError::TypeError {
                msg: format!("Expected partial, got: {}", self.to_polar()),
                stack_trace: None, // @TODO
            }),
        }
    }

    pub fn as_call(&self) -> Result<&Call, error::RuntimeError> {
        match self {
            Value::Call(pred) => Ok(pred),
            _ => Err(error::RuntimeError::TypeError {
                msg: format!("Expected call, got: {}", self.to_polar()),
                stack_trace: None, // @TODO
            }),
        }
    }

    pub fn as_pattern(&self) -> Result<&Pattern, error::RuntimeError> {
        match self {
            Value::Pattern(p) => Ok(p),
            _ => Err(error::RuntimeError::TypeError {
                msg: format!("Expected pattern, got: {}", self.to_polar()),
                stack_trace: None, // @TODO
            }),
        }
    }

    pub fn is_ground(&self) -> bool {
        match self {
            Value::Call(_)
            | Value::ExternalInstance(_)
            | Value::Variable(_)
            | Value::RestVariable(_)
            | Value::Partial(_) => false,
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
    pub fn new_from_ffi(value: Value) -> Self {
        Self {
            source_info: SourceInfo::Ffi,
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
    pub fn variables<'set>(&self, vars: &'set mut HashSet<Symbol>) {
        struct VariableVisitor<'set> {
            vars: &'set mut HashSet<Symbol>,
        }

        impl<'set> VariableVisitor<'set> {
            fn new(vars: &'set mut HashSet<Symbol>) -> Self {
                Self { vars }
            }
        }

        impl<'set> Visitor for VariableVisitor<'set> {
            fn visit_variable(&mut self, v: &Symbol) {
                self.vars.insert(v.clone());
            }
        }

        walk_term(&mut VariableVisitor::new(vars), self);
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
