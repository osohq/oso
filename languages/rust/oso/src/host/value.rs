use polar_core::terms::*;
use std::collections::btree_map::BTreeMap;
use std::collections::hash_map::HashMap;

// Should we call it something else?
#[derive(Clone, Debug, PartialEq)]
pub enum PolarValue {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Map(HashMap<String, PolarValue>),
    List(Vec<PolarValue>),
    Variable(String),
}

impl PolarValue {
    pub fn from_term(term: &Term) -> crate::Result<Self> {
        let val = match term.value() {
            Value::Number(Numeric::Integer(i)) => PolarValue::Integer(*i),
            Value::Number(Numeric::Float(f)) => PolarValue::Float(*f),
            Value::String(s) => PolarValue::String(s.clone()),
            Value::Boolean(b) => PolarValue::Boolean(*b),
            Value::Dictionary(dict) => {
                let mut map = HashMap::new();
                for (k, v) in &dict.fields {
                    let key = k.0.clone();
                    let value = PolarValue::from_term(v)?;
                    map.insert(key, value);
                }
                PolarValue::Map(map)
            }
            Value::List(l) => {
                let mut list = vec![];
                for t in l {
                    list.push(PolarValue::from_term(t)?);
                }
                PolarValue::List(list)
            }
            Value::Variable(Symbol(sym)) => PolarValue::Variable(sym.clone()),
            _ => return Err(crate::OsoError::FromPolar),
        };
        Ok(val)
    }

    pub fn to_term(&self) -> Term {
        let value = match self {
            PolarValue::Integer(i) => Value::Number(Numeric::Integer(*i)),
            PolarValue::Float(f) => Value::Number(Numeric::Float(*f)),
            PolarValue::String(s) => Value::String(s.clone()),
            PolarValue::Boolean(b) => Value::Boolean(*b),
            PolarValue::Map(map) => {
                let mut dict = Dictionary::new();
                for (k, v) in map {
                    let key = Symbol(k.clone());
                    let value = v.to_term();
                    dict.fields.insert(key, value);
                }
                Value::Dictionary(dict)
            }
            PolarValue::List(l) => {
                let mut list = vec![];
                for v in l {
                    list.push(v.to_term())
                }
                Value::List(list)
            }
            PolarValue::Variable(s) => Value::Variable(Symbol(s.clone())),
        };
        Term::new_from_ffi(value)
    }
}
