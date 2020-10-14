use std::collections::hash_map::HashMap;
use std::convert::TryFrom;

use polar_core::terms::*;

use crate::host::Host;
use crate::PolarClass;

use crate::errors::TypeError;

// Should we call it something else?
#[derive(Clone, Debug)]
pub enum PolarValue {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Map(HashMap<String, PolarValue>),
    List(Vec<PolarValue>),
    Variable(String),
    Instance(crate::host::class::Instance),
}

impl PolarValue {
    pub fn new_from_instance<T>(instance: T) -> Self
    where
        T: Send + Sync + 'static,
    {
        Self::Instance(crate::host::class::Instance::new(instance))
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
            _ => {
                return Err(TypeError {
                    expected: "Unsupported Value Type".to_owned(),
                }
                .user())
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
                    repr: Some(std::any::type_name::<Self>().to_owned()),
                    instance_id: id,
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
                    Err(TypeError {
                        expected: "Integer".to_owned(),
                    }
                    .user())
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

impl<T> FromPolarValue for T
where
    T: 'static + Clone + PolarClass,
{
    fn from_polar_value(val: PolarValue) -> crate::Result<Self> {
        if let PolarValue::Instance(instance) = val {
            Ok(instance.downcast::<T>()?.clone())
        } else {
            Err(TypeError {
                expected: "Instance".to_owned(),
            }
            .user())
        }
    }
}

impl FromPolarValue for f64 {
    fn from_polar_value(val: PolarValue) -> crate::Result<Self> {
        if let PolarValue::Float(f) = val {
            Ok(f)
        } else {
            Err(TypeError {
                expected: "Float".to_owned(),
            }
            .user())
        }
    }
}

impl FromPolarValue for String {
    fn from_polar_value(val: PolarValue) -> crate::Result<Self> {
        if let PolarValue::String(s) = val {
            Ok(s)
        } else {
            Err(TypeError {
                expected: "String".to_owned(),
            }
            .user())
        }
    }
}

impl FromPolarValue for bool {
    fn from_polar_value(val: PolarValue) -> crate::Result<Self> {
        if let PolarValue::Boolean(b) = val {
            Ok(b)
        } else {
            Err(TypeError {
                expected: "Boolean".to_owned(),
            }
            .user())
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
            Err(TypeError {
                expected: "Map".to_owned(),
            }
            .user())
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
            Err(TypeError {
                expected: "List".to_owned(),
            }
            .user())
        }
    }
}

// well, you can't do this
// impl<U: FromPolarValue> TryFrom<U> for PolarValue {
//     type Error = crate::OsoError;

//     fn try_from(v: PolarValue) -> Result<Self, Self::Error> {
//         U::from_polar_value(v)
//     }
// }

// so I have to do this
macro_rules! try_from_polar {
    ($i:ty) => {
        impl TryFrom<PolarValue> for $i {
            type Error = crate::OsoError;

            fn try_from(v: PolarValue) -> Result<Self, Self::Error> {
                Self::from_polar_value(v)
            }
        }
    };
}

try_from_polar!(u8);
try_from_polar!(i8);
try_from_polar!(u16);
try_from_polar!(i16);
try_from_polar!(u32);
try_from_polar!(i32);
try_from_polar!(i64);
try_from_polar!(f64);
try_from_polar!(String);
try_from_polar!(bool);

impl<T: FromPolarValue> TryFrom<PolarValue> for HashMap<String, T> {
    type Error = crate::OsoError;

    fn try_from(v: PolarValue) -> Result<Self, Self::Error> {
        Self::from_polar_value(v)
    }
}

impl<T: FromPolarValue> TryFrom<PolarValue> for Vec<T> {
    type Error = crate::OsoError;

    fn try_from(v: PolarValue) -> Result<Self, Self::Error> {
        Self::from_polar_value(v)
    }
}
