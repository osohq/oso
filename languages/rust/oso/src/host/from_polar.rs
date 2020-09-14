//! Trait and implementations of `FromPolar` for converting from
//! Polar types back to Rust types.

use polar_core::terms::*;

use std::collections::HashMap;
use std::convert::TryFrom;

use super::class::Instance;
use super::{Host, HostClass};

pub trait FromPolar: Sized {
    fn from_polar(term: &Term, host: &mut Host) -> crate::Result<Self>;

    fn from_polar_list(terms: &[Term], host: &mut Host) -> crate::Result<Self> {
        assert_eq!(terms.len(), 1);
        Self::from_polar(&terms[0], host)
    }
}

impl<C: 'static + Clone + HostClass> FromPolar for C {
    fn from_polar(term: &Term, host: &mut Host) -> crate::Result<Self> {
        match term.value() {
            Value::ExternalInstance(ExternalInstance { instance_id, .. }) => host
                .get_instance(*instance_id)
                .and_then(|instance| instance.instance.downcast_ref::<C>().cloned())
                .ok_or_else(|| crate::OsoError::FromPolar),
            _ => Err(crate::OsoError::FromPolar),
        }
    }
}

impl FromPolar for bool {
    fn from_polar(term: &Term, _host: &mut Host) -> crate::Result<Self> {
        if let Value::Boolean(b) = term.value() {
            Ok(*b)
        } else {
            Err(crate::OsoError::FromPolar)
        }
    }
}

macro_rules! polar_to_int {
    ($i:ty) => {
        impl FromPolar for $i {
            fn from_polar(term: &Term, _host: &mut Host) -> crate::Result<Self> {
                if let Value::Number(Numeric::Integer(i)) = term.value() {
                    <$i>::try_from(*i).map_err(|_| crate::OsoError::FromPolar)
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

impl FromPolar for f64 {
    fn from_polar(term: &Term, _host: &mut Host) -> crate::Result<Self> {
        if let Value::Number(Numeric::Float(f)) = term.value() {
            Ok(*f)
        } else {
            Err(crate::OsoError::FromPolar)
        }
    }
}

impl FromPolar for String {
    fn from_polar(term: &Term, _host: &mut Host) -> crate::Result<Self> {
        if let Value::String(s) = term.value() {
            Ok(s.to_string())
        } else {
            Err(crate::OsoError::FromPolar)
        }
    }
}

impl<T: FromPolar> FromPolar for Vec<T> {
    fn from_polar(term: &Term, host: &mut Host) -> crate::Result<Self> {
        if let Value::List(l) = term.value() {
            l.iter().map(|t| T::from_polar(t, host)).collect()
        } else {
            Err(crate::OsoError::FromPolar)
        }
    }

    fn from_polar_list(terms: &[Term], host: &mut Host) -> crate::Result<Self> {
        terms.iter().map(|t| T::from_polar(t, host)).collect()
    }
}

impl<T: FromPolar> FromPolar for HashMap<String, T> {
    fn from_polar(term: &Term, host: &mut Host) -> crate::Result<Self> {
        if let Value::Dictionary(dict) = term.value() {
            dict.fields
                .iter()
                .map(|(k, v)| T::from_polar(v, host).map(|v| (k.0.clone(), v)))
                .collect()
        } else {
            Err(crate::OsoError::FromPolar)
        }
    }
}

impl FromPolar for Value {
    fn from_polar(term: &Term, _host: &mut Host) -> crate::Result<Self> {
        Ok(term.value().clone())
    }
}

impl FromPolar for Instance {
    fn from_polar(term: &Term, host: &mut Host) -> crate::Result<Self> {
        let instance = match term.value().clone() {
            Value::Boolean(b) => host
                .get_class_from_type::<bool>()
                .unwrap()
                .cast_to_instance(b),
            Value::Number(Numeric::Integer(i)) => host
                .get_class_from_type::<i64>()
                .unwrap()
                .cast_to_instance(i),
            Value::Number(Numeric::Float(f)) => host
                .get_class_from_type::<f64>()
                .unwrap()
                .cast_to_instance(f),
            Value::List(v) => host
                .get_class_from_type::<Vec<Term>>()
                .unwrap()
                .cast_to_instance(v),
            Value::String(s) => host
                .get_class_from_type::<String>()
                .unwrap()
                .cast_to_instance(s),
            Value::Dictionary(d) => host
                .get_class_from_type::<HashMap<Symbol, Term>>()
                .unwrap()
                .cast_to_instance(d.fields),
            Value::ExternalInstance(ExternalInstance { instance_id, .. }) => host
                .get_instance(instance_id)
                .expect("instance not found")
                .clone(),
            v => {
                tracing::warn!(value = ?v, "invalid conversion attempted");
                return Err(crate::OsoError::FromPolar);
            }
        };
        Ok(instance)
    }
}

impl FromPolar for () {
    fn from_polar(term: &Term, _host: &mut Host) -> crate::Result<Self> {
        if let Value::List(l) = term.value() {
            if l.is_empty() {
                Ok(())
            } else {
                Err(crate::OsoError::FromPolar)
            }
        } else {
            Err(crate::OsoError::FromPolar)
        }
    }

    fn from_polar_list(terms: &[Term], _host: &mut Host) -> crate::Result<Self> {
        if terms.is_empty() {
            Ok(())
        } else {
            Err(crate::OsoError::FromPolar)
        }
    }
}

impl<A> FromPolar for (A,)
where
    A: FromPolar,
{
    fn from_polar(_term: &Term, _host: &mut Host) -> crate::Result<Self> {
        Err(crate::OsoError::FromPolar)
    }

    fn from_polar_list(terms: &[Term], host: &mut Host) -> crate::Result<Self> {
        if terms.len() == 1 {
            A::from_polar(&terms[0], host).map(|a| (a,))
        } else {
            Err(crate::OsoError::FromPolar)
        }
    }
}

impl<A, B> FromPolar for (A, B)
where
    A: FromPolar,
    B: FromPolar,
{
    fn from_polar(_term: &Term, _host: &mut Host) -> crate::Result<Self> {
        Err(crate::OsoError::FromPolar)
    }

    fn from_polar_list(terms: &[Term], host: &mut Host) -> crate::Result<Self> {
        if terms.len() == 2 {
            let a = A::from_polar(&terms[0], host)?;
            let b = B::from_polar(&terms[1], host)?;
            Ok((a, b))
        } else {
            Err(crate::OsoError::FromPolar)
        }
    }
}
