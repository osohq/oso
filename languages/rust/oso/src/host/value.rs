use polar_core::terms::*;
use std::collections::btree_map::BTreeMap;
use std::collections::hash_map::HashMap;
use std::convert::TryFrom;

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

pub trait FromPolarValue: Clone + Sized + 'static {
    fn from_polar_value(val: PolarValue) -> crate::Result<Self>;
}

impl FromPolarValue for PolarValue {
    fn from_polar_value(val: PolarValue) -> crate::Result<Self> {
        Ok(val)
    }
}

macro_rules! polar_to_int {
    ($i:ty) => {
        impl FromPolarValue for $i {
            fn from_polar_value(val: PolarValue) -> crate::Result<Self> {
                if let PolarValue::Integer(i) = val {
                    <$i>::try_from(i).map_err(|_| crate::OsoError::FromPolar)
                } else {
                    Err(crate::OsoError::FromPolar)
                }
            }
        }
    };
}

polar_to_int!(u8);
polar_to_int!(i8);
polar_to_int!(u16);
polar_to_int!(i16);
polar_to_int!(u32);
polar_to_int!(i32);
polar_to_int!(i64);

impl FromPolarValue for f64 {
    fn from_polar_value(val: PolarValue) -> crate::Result<Self> {
        if let PolarValue::Float(f) = val {
            Ok(f)
        } else {
            Err(crate::OsoError::FromPolar)
        }
    }
}

impl FromPolarValue for String {
    fn from_polar_value(val: PolarValue) -> crate::Result<Self> {
        if let PolarValue::String(s) = val {
            Ok(s)
        } else {
            Err(crate::OsoError::FromPolar)
        }
    }
}

impl FromPolarValue for bool {
    fn from_polar_value(val: PolarValue) -> crate::Result<Self> {
        if let PolarValue::Boolean(b) = val {
            Ok(b)
        } else {
            Err(crate::OsoError::FromPolar)
        }
    }
}

impl<T: FromPolarValue> FromPolarValue for HashMap<String, T> {
    fn from_polar_value(val: PolarValue) -> crate::Result<Self> {
        if let PolarValue::Map(map) = val {
            let mut result = HashMap::new();
            for (k, v) in map {
                let val = T::from_polar_value(v)?;
                result.insert(k, val);
            }
            Ok(result)
        } else {
            Err(crate::OsoError::FromPolar)
        }
    }
}

impl<T: FromPolarValue> FromPolarValue for Vec<T> {
    fn from_polar_value(val: PolarValue) -> crate::Result<Self> {
        if let PolarValue::List(l) = val {
            let mut result = vec![];
            for v in l {
                result.push(T::from_polar_value(v)?);
            }
            Ok(result)
        } else {
            Err(crate::OsoError::FromPolar)
        }
    }
}
