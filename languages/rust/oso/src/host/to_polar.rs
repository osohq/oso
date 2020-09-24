//! Trait and implementations of `ToPolar` for converting from
//! Rust types back to Polar types.

use impl_trait_for_tuples::*;
use polar_core::terms::*;

use std::collections::HashMap;
use std::iter;

use super::Host;
use crate::host::Instance;

/// Convert Rust types to Polar types.
///
/// This trait is automatically implemented for any
/// type that implements the `PolarClass` marker trait,
/// which should be preferred.
///
/// This is also implemented automatically using the
/// `#[derive(PolarClass)]` macro.
///
/// For non-primitive types, the instance will be stored
/// on the provided `Host`.
/// ## Trait bounds
///
/// `ToPolar` requires types to be `Send + Sync`, since it
/// is possible to store a `ToPolar` value on an `Oso` instance
/// which can be shared between threads.
///
/// `ToPolar` implementors must also be concrete, sized types without
/// any borrows.
pub trait ToPolar: Send + Sync + Sized + 'static {
    fn to_polar_value(self, host: &mut Host) -> Value {
        let instance = Instance::new(self);
        let instance = host.cache_instance(instance, None);
        Value::ExternalInstance(ExternalInstance {
            constructor: None,
            repr: Some(std::any::type_name::<Self>().to_owned()),
            instance_id: instance,
        })
    }

    fn to_polar(self, host: &mut Host) -> Term {
        Term::new_from_ffi(self.to_polar_value(host))
    }
}

impl<C: crate::PolarClass + Send + Sync> ToPolar for C {
    fn to_polar_value(self, host: &mut Host) -> Value {
        let instance = Instance::new(self);
        let instance = host.cache_instance(instance, None);
        if host.get_class_from_type::<Self>().is_err() {
            let class = Self::get_polar_class();
            let name = Symbol(class.name.clone());
            tracing::info!("class {} not previously registered, doing so now", name.0);
            // If we hit this error its because somehow we didn't find the class, and yet
            // we also weren't able to register the class because the name already exists.
            // TODO: can we handle this without panicking?
            host.cache_class(class, name.clone())
                .expect("failed to register a class that we thought was previously unregistered");
        }
        Value::ExternalInstance(ExternalInstance {
            constructor: None,
            repr: Some(std::any::type_name::<Self>().to_owned()),
            instance_id: instance,
        })
    }
}

mod private {
    /// Prevents implementations of `ToPolarList` outside of this crate
    pub trait Sealed {}
}

pub trait ToPolarList: private::Sealed {
    fn to_polar_list(self, host: &mut Host) -> Vec<Term>
    where
        Self: Sized;
}

#[impl_for_tuples(16)]
#[tuple_types_custom_trait_bound(ToPolar + 'static)]
impl private::Sealed for Tuple {}

impl ToPolarList for () {
    fn to_polar_list(self, _host: &mut Host) -> Vec<Term> {
        Vec::new()
    }
}

#[impl_for_tuples(1, 16)]
#[tuple_types_custom_trait_bound(ToPolar + 'static)]
impl ToPolarList for Tuple {
    fn to_polar_list(self, host: &mut Host) -> Vec<Term> {
        let mut result = Vec::new();
        for_tuples!(
            #( result.push(self.Tuple.to_polar(host)); )*
        );
        result
    }
}

impl ToPolar for bool {
    fn to_polar_value(self, _host: &mut Host) -> Value {
        Value::Boolean(self)
    }
}

macro_rules! int_to_polar {
    ($i:ty) => {
        impl ToPolar for $i {
            fn to_polar_value(self, _host: &mut Host) -> Value {
                Value::Number(Numeric::Integer(self.into()))
            }
        }
    };
}

int_to_polar!(u8);
int_to_polar!(i8);
int_to_polar!(u16);
int_to_polar!(i16);
int_to_polar!(u32);
int_to_polar!(i32);
int_to_polar!(i64);

macro_rules! float_to_polar {
    ($i:ty) => {
        impl ToPolar for $i {
            fn to_polar_value(self, _host: &mut Host) -> Value {
                Value::Number(Numeric::Float(self.into()))
            }
        }
    };
}

float_to_polar!(f32);
float_to_polar!(f64);

impl ToPolar for String {
    fn to_polar_value(self, _host: &mut Host) -> Value {
        Value::String(self)
    }
}

impl ToPolar for &'static str {
    fn to_polar_value(self, _host: &mut Host) -> Value {
        Value::String(self.to_string())
    }
}

impl<T: ToPolar> ToPolar for Vec<T> {
    fn to_polar_value(self, host: &mut Host) -> Value {
        Value::List(self.into_iter().map(|v| v.to_polar(host)).collect())
    }
}

impl<T: ToPolar> ToPolar for HashMap<String, T> {
    fn to_polar_value(self, host: &mut Host) -> Value {
        Value::Dictionary(Dictionary {
            fields: self
                .into_iter()
                .map(|(k, v)| (Symbol(k), v.to_polar(host)))
                .collect(),
        })
    }
}

impl ToPolar for Value {
    fn to_polar_value(self, _host: &mut Host) -> Value {
        self
    }
}

pub type PolarResultIter = Box<dyn Iterator<Item = Result<Term, crate::OsoError>> + 'static>;

// Trait for the return value of class methods.
// This allows us to return polar values, as well as options and results of polar values.
pub trait ToPolarResults {
    fn to_polar_results(self, host: &mut Host) -> PolarResultIter;
}

impl<C: 'static + Sized + ToPolar> ToPolarResults for C {
    fn to_polar_results(self, host: &mut Host) -> PolarResultIter {
        Box::new(iter::once(Ok(self.to_polar(host))))
    }
}

impl<C: ToPolarResults, E: ToString> ToPolarResults for Result<C, E> {
    fn to_polar_results(self, host: &mut Host) -> PolarResultIter {
        match self {
            Ok(result) => result.to_polar_results(host),
            Err(e) => Box::new(iter::once(Err(crate::OsoError::Custom {
                message: e.to_string(),
            }))),
        }
    }
}

impl<C: ToPolarResults> ToPolarResults for Option<C> {
    fn to_polar_results(self, host: &mut Host) -> PolarResultIter {
        self.map_or_else(
            || Box::new(std::iter::empty()) as PolarResultIter,
            |c| c.to_polar_results(host),
        )
    }
}

pub struct PolarIter<I, Iter>
where
    I: ToPolarResults + 'static,
    Iter: std::iter::Iterator<Item = I> + Sized + 'static,
{
    pub iter: Iter,
}

impl<I: ToPolarResults + 'static, Iter: std::iter::Iterator<Item = I> + Sized + 'static>
    ToPolarResults for PolarIter<I, Iter>
{
    fn to_polar_results(self, host: &mut Host) -> PolarResultIter {
        Box::new(
            self.iter
                .flat_map(|i| i.to_polar_results(host))
                .collect::<Vec<crate::Result<Term>>>()
                .into_iter(),
        ) as PolarResultIter
    }
}
