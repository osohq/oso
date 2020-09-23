#![allow(clippy::many_single_char_names, clippy::type_complexity)]
//! Trait and implementations of `FromPolar` for converting from
//! Polar types back to Rust types.

use impl_trait_for_tuples::*;
use polar_core::terms::*;

use std::collections::HashMap;
use std::convert::TryFrom;

use super::class::Instance;
use super::Host;
use crate::PolarClass;

/// Convert Polar types to Rust types.
///
/// This trait is automatically implemented for any
/// type that implements the `PolarClass` trait,
/// which should be preferred.
///
/// This is also implemented automatically using the
/// `#[derive(PolarClass)]` macro.
///
/// ## Trait bounds
///
/// Currently `FromPolar` requires `Clone` because we can only
/// get a borrowed value back from oso. In the future, this could
/// be updated to return borrowed data instead.
///
/// `FromPolar` also requires types to be `Send + Sync`, since it
/// is possible to store a `FromPolar` value on an `Oso` instance
/// which can be shared between threads
///
/// `FromPolar` implementors must also be concrete, sized types without
/// any borrows.
pub trait FromPolar: Clone + Send + Sync + Sized + 'static {
    fn from_polar(term: &Term, host: &Host) -> crate::Result<Self> {
        match term.value() {
            Value::ExternalInstance(ExternalInstance { instance_id, .. }) => host
                .get_instance(*instance_id)
                .ok_or_else(|| crate::OsoError::FromPolar)
                .and_then(|instance| {
                    instance
                        .downcast::<Self>()
                        .map_err(|e| e.invariant().into())
                })
                .map(Clone::clone),
            _ => Err(crate::OsoError::FromPolar),
        }
    }
}

impl<C: 'static + Clone + Send + Sync + PolarClass> FromPolar for C {}

mod private {
    /// Prevents implementations of `FromPolarList` outside of this crate
    pub trait Sealed {}
}

pub trait FromPolarList: private::Sealed {
    fn from_polar_list(terms: &[Term], host: &Host) -> crate::Result<Self>
    where
        Self: Sized;
}

impl FromPolar for bool {
    fn from_polar(term: &Term, _host: &Host) -> crate::Result<Self> {
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
            fn from_polar(term: &Term, _host: &Host) -> crate::Result<Self> {
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
    fn from_polar(term: &Term, _host: &Host) -> crate::Result<Self> {
        if let Value::Number(Numeric::Float(f)) = term.value() {
            Ok(*f)
        } else {
            Err(crate::OsoError::FromPolar)
        }
    }
}

impl FromPolar for String {
    fn from_polar(term: &Term, _host: &Host) -> crate::Result<Self> {
        if let Value::String(s) = term.value() {
            Ok(s.to_string())
        } else {
            Err(crate::OsoError::FromPolar)
        }
    }
}

impl<T: FromPolar> FromPolar for Vec<T> {
    fn from_polar(term: &Term, host: &Host) -> crate::Result<Self> {
        if let Value::List(l) = term.value() {
            l.iter().map(|t| T::from_polar(t, host)).collect()
        } else {
            Err(crate::OsoError::FromPolar)
        }
    }
}

impl<T: FromPolar> FromPolar for HashMap<String, T> {
    fn from_polar(term: &Term, host: &Host) -> crate::Result<Self> {
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
    fn from_polar(term: &Term, _host: &Host) -> crate::Result<Self> {
        Ok(term.value().clone())
    }
}

impl FromPolar for Instance {
    fn from_polar(term: &Term, host: &Host) -> crate::Result<Self> {
        // We need to handle converting all value variants to an
        // instance so that we can use the `Class` mechanism to
        // handle methods on them
        let instance = match &term.value() {
            Value::Boolean(b) => Instance::new(*b),
            Value::Number(Numeric::Integer(i)) => Instance::new(*i),
            Value::Number(Numeric::Float(f)) => Instance::new(*f),
            Value::List(v) => Instance::new(v.clone()),
            Value::String(s) => Instance::new(s.clone()),
            Value::Dictionary(d) => Instance::new(d.fields.clone()),
            Value::ExternalInstance(ExternalInstance { instance_id, .. }) => host
                .get_instance(*instance_id)
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

#[impl_for_tuples(16)]
#[tuple_types_custom_trait_bound(FromPolar)]
impl FromPolarList for Tuple {
    fn from_polar_list(terms: &[Term], host: &Host) -> crate::Result<Self> {
        let mut iter = terms.iter();
        Ok((for_tuples!(
            #( Tuple::from_polar(iter.next().expect("not enough arguments provided"), host)? ),*
        )))
    }
}

#[impl_for_tuples(16)]
#[tuple_types_custom_trait_bound(FromPolar)]
impl private::Sealed for Tuple {}
