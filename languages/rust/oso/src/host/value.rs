use polar_core::terms::*;
use std::collections::hash_map::HashMap;

use crate::host::{Host, Instance};

/// An enum of the possible value types that can be
/// sent to/from Polar.
///
/// All variants except `Instance` represent types that can
/// be used natively in Polar.
/// Any other types can be wrapped using `PolarValue::new_from_instance`.
/// If the instance has a registered `Class`, then this can be used
/// from the policy too.
#[derive(Clone, Debug)]
pub enum PolarValue {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Map(HashMap<String, PolarValue>),
    List(Vec<PolarValue>),
    Variable(String),
    Instance(Instance),
}

impl PartialEq for PolarValue {
    fn eq(&self, other: &PolarValue) -> bool {
        match (self, other) {
            (PolarValue::Boolean(b1), PolarValue::Boolean(b2)) => b1 == b2,
            (PolarValue::Float(f1), PolarValue::Float(f2)) => f1 == f2,
            (PolarValue::Integer(i1), PolarValue::Integer(i2)) => i1 == i2,
            (PolarValue::List(l1), PolarValue::List(l2)) => l1 == l2,
            (PolarValue::Map(m1), PolarValue::Map(m2)) => m1 == m2,
            (PolarValue::String(s1), PolarValue::String(s2)) => s1 == s2,
            _ => false,
        }
    }
}

impl PolarValue {
    /// Create a `PolarValue::Instance` from any type.
    pub fn new_from_instance<T>(instance: T) -> Self
    where
        T: Send + Sync + 'static,
    {
        Self::Instance(Instance::new(instance))
    }

    pub(crate) fn from_term(term: &Term, host: &Host) -> crate::Result<Self> {
        let val = match term.value() {
            Value::Number(Numeric::Integer(i)) => PolarValue::Integer(*i),
            Value::Number(Numeric::Float(f)) => PolarValue::Float(*f),
            Value::String(s) => PolarValue::String(s.clone()),
            Value::Boolean(b) => PolarValue::Boolean(*b),
            Value::Dictionary(dict) => {
                let mut map = HashMap::new();
                for (k, v) in &dict.fields {
                    let key = k.0.clone();
                    let value = PolarValue::from_term(v, host)?;
                    map.insert(key, value);
                }
                PolarValue::Map(map)
            }
            Value::ExternalInstance(instance) => {
                PolarValue::Instance(host.get_instance(instance.instance_id)?.clone())
            }
            Value::List(l) => {
                let mut list = vec![];
                for t in l {
                    list.push(PolarValue::from_term(t, host)?);
                }
                PolarValue::List(list)
            }
            Value::Variable(Symbol(sym)) => PolarValue::Variable(sym.clone()),
            Value::Expression(_) => {
                return Err(crate::OsoError::Custom {
                    message: r#"
Received Expression from Polar VM. The Expression type is not yet supported in this language.

This may mean you performed an operation in your policy over an unbound variable.
                        "#
                    .to_owned(),
                })
            }
            _ => {
                return Err(crate::OsoError::Custom {
                    message: "Unsupported value type".to_owned(),
                })
            }
        };
        Ok(val)
    }

    pub(crate) fn to_term(&self, host: &mut Host) -> Term {
        let value = match self {
            PolarValue::Integer(i) => Value::Number(Numeric::Integer(*i)),
            PolarValue::Float(f) => Value::Number(Numeric::Float(*f)),
            PolarValue::String(s) => Value::String(s.clone()),
            PolarValue::Boolean(b) => Value::Boolean(*b),
            PolarValue::Map(map) => {
                let mut dict = Dictionary::new();
                for (k, v) in map {
                    let key = Symbol(k.clone());
                    let value = v.to_term(host);
                    dict.fields.insert(key, value);
                }
                Value::Dictionary(dict)
            }
            PolarValue::Instance(instance) => {
                let id = host.cache_instance(instance.clone(), None);
                Value::ExternalInstance(ExternalInstance {
                    constructor: None,
                    instance_id: id,
                    repr: Some(std::any::type_name::<Self>().to_owned()),
                    class_repr: Some(std::any::type_name::<Self>().to_owned()),
                    class_id: None,
                })
            }
            PolarValue::List(l) => {
                let mut list = vec![];
                for v in l {
                    list.push(v.to_term(host))
                }
                Value::List(list)
            }
            PolarValue::Variable(s) => Value::Variable(Symbol(s.clone())),
        };
        Term::new_from_ffi(value)
    }

    pub fn type_name(&self) -> PolarValueType {
        match self {
            PolarValue::Integer(_) => PolarValueType::Integer,
            PolarValue::Float(_) => PolarValueType::Float,
            PolarValue::String(_) => PolarValueType::String,
            PolarValue::Boolean(_) => PolarValueType::Boolean,
            PolarValue::Map(_) => PolarValueType::Map,
            PolarValue::List(_) => PolarValueType::List,
            PolarValue::Variable(_) => PolarValueType::Variable,
            PolarValue::Instance(i) => PolarValueType::Instance(i.debug_name()),
        }
    }
}

#[derive(Clone, Debug)]
pub enum PolarValueType {
    Integer,
    Float,
    String,
    Boolean,
    Map,
    List,
    Variable,
    Instance(&'static str),
}

impl std::fmt::Display for PolarValueType {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PolarValueType::Instance(name) => write!(fmt, "Instance<{}>", name),
            _ => std::fmt::Debug::fmt(self, fmt),
        }
    }
}
