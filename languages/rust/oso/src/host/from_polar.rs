#![allow(clippy::many_single_char_names, clippy::type_complexity)]
//! Trait and implementations of `FromPolar` for converting from
//! Polar types back to Rust types.

use impl_trait_for_tuples::*;
use polar_core::terms::{self, Term, Value, Numeric};

use super::class::Instance;
use super::Host;
use crate::errors::TypeError;

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
pub trait FromPolar: Clone + Sized + 'static {
    fn from_polar(term: &Term, host: &Host) -> crate::Result<Self> {
        let wrong_value = match term.value() {
            terms::Value::ExternalInstance(terms::ExternalInstance { instance_id, .. }) => return host
                .get_instance(*instance_id)
                .and_then(|instance| {
                    instance
                        .downcast::<Self>(Some(&host))
                         // TODO (dhatch): This might be user.
                        .map_err(|e| e.invariant().into())
                })
                .map(Clone::clone),
            val => val
        };

        let expected = host.get_class_from_type::<Self>()
            .map(|class| class.name.clone())
            .ok()
            .unwrap_or_else(|| std::any::type_name::<Self>().to_owned());

        // TODO (dhatch): Should this operate on our oso value type instead of
        // the polar core value type?
        let got = match wrong_value {
            Value::Number(Numeric::Integer(_)) => Some("Integer"),
            Value::Number(Numeric::Float(_)) => Some("Float"),
            Value::String(_) => Some("String"),
            Value::Boolean(_) => Some("Boolean"),
            Value::List(_) => Some("List"),
            Value::Dictionary(_) => Some("Dictionary"),
            Value::Variable(_) => Some("Variable"),
            Value::Call(_) => Some("Predicate"),
            // Other types are unexpected and therefore do not make their
            // way into the error message.
            _ => None
        };

        let mut type_error = TypeError::expected(expected);

        if let Some(got) = got {
            type_error = type_error.got(got.to_owned());
        }

        Err(type_error.user())
    }
}

mod private {
    /// Prevents implementations of `FromPolarList` outside of this crate
    pub trait Sealed {}
}

pub trait FromPolarList: private::Sealed {
    fn from_polar_list(terms: &[Term], host: &Host) -> crate::Result<Self>
    where
        Self: Sized;
}

impl<T: crate::FromPolarValue> FromPolar for T {
    fn from_polar(term: &Term, host: &Host) -> crate::Result<Self> {
        T::from_polar_value(crate::PolarValue::from_term(term, host)?)
    }
}

impl FromPolar for Instance {
    fn from_polar(term: &Term, host: &Host) -> crate::Result<Self> {
        // We need to handle converting all value variants to an
        // instance so that we can use the `Class` mechanism to
        // handle methods on them
        let instance = match &term.value() {
            terms::Value::Boolean(b) => Instance::new(*b),
            terms::Value::Number(terms::Numeric::Integer(i)) => Instance::new(*i),
            terms::Value::Number(terms::Numeric::Float(f)) => Instance::new(*f),
            terms::Value::List(v) => Instance::new(v.clone()),
            terms::Value::String(s) => Instance::new(s.clone()),
            terms::Value::Dictionary(d) => Instance::new(d.fields.clone()),
            terms::Value::ExternalInstance(terms::ExternalInstance { instance_id, .. }) => host
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
        let result = Ok((for_tuples!(
            #( Tuple::from_polar(iter.next().expect("not enough arguments provided"), host)? ),*
        )));

        if iter.len() > 0 {
            // TODO (dhatch): Debug this!!!
            tracing::warn!("Remaining items in iterator after conversion.");
            for item in iter {
                tracing::trace!("Remaining item {}", item);
            }

            return Err(crate::OsoError::FromPolar);
        }

        result
    }
}

#[impl_for_tuples(16)]
#[tuple_types_custom_trait_bound(FromPolar)]
impl private::Sealed for Tuple {}
