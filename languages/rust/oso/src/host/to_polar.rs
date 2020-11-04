//! Trait and implementations of `ToPolar` for converting from
//! Rust types back to Polar types.

use impl_trait_for_tuples::*;

use std::collections::HashMap;
use std::iter;

use crate::PolarValue;

lazy_static::lazy_static! {
    pub static ref DEFAULT_CLASSES: std::sync::Arc<std::sync::Mutex<HashMap<std::any::TypeId, super::Class>>> = Default::default();
}

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
    fn to_polar(self) -> PolarValue {
        PolarValue::new_from_instance(self)
    }
}

impl<C: crate::PolarClass + Send + Sync> ToPolar for C {
    fn to_polar(self) -> PolarValue {
        DEFAULT_CLASSES
            .lock()
            .unwrap()
            .entry(std::any::TypeId::of::<C>())
            .or_insert_with(C::get_polar_class);

        PolarValue::new_from_instance(self)
    }
}

mod private {
    /// Prevents implementations of `ToPolarList` outside of this crate
    pub trait Sealed {}
}

pub trait ToPolarList: private::Sealed {
    fn to_polar_list(self) -> Vec<PolarValue>
    where
        Self: Sized;
}

#[impl_for_tuples(16)]
#[tuple_types_custom_trait_bound(ToPolar + 'static)]
impl private::Sealed for Tuple {}

impl ToPolarList for () {
    fn to_polar_list(self) -> Vec<PolarValue> {
        Vec::new()
    }
}

#[impl_for_tuples(1, 16)]
#[tuple_types_custom_trait_bound(ToPolar + 'static)]
impl ToPolarList for Tuple {
    fn to_polar_list(self) -> Vec<PolarValue> {
        let mut result = Vec::new();
        for_tuples!(
            #( result.push(self.Tuple.to_polar()); )*
        );
        result
    }
}

impl ToPolar for bool {
    fn to_polar(self) -> PolarValue {
        PolarValue::Boolean(self)
    }
}

macro_rules! int_to_polar {
    ($i:ty) => {
        impl ToPolar for $i {
            fn to_polar(self) -> PolarValue {
                PolarValue::Integer(self.into())
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
            fn to_polar(self) -> PolarValue {
                PolarValue::Float(self.into())
            }
        }
    };
}

float_to_polar!(f32);
float_to_polar!(f64);

impl ToPolar for String {
    fn to_polar(self) -> PolarValue {
        PolarValue::String(self)
    }
}

impl ToPolar for &'static str {
    fn to_polar(self) -> PolarValue {
        PolarValue::String(self.to_string())
    }
}

impl<T: ToPolar> ToPolar for Vec<T> {
    fn to_polar(self) -> PolarValue {
        PolarValue::List(self.into_iter().map(|v| v.to_polar()).collect())
    }
}

impl<T: ToPolar> ToPolar for HashMap<String, T> {
    fn to_polar(self) -> PolarValue {
        PolarValue::Map(self.into_iter().map(|(k, v)| (k, v.to_polar())).collect())
    }
}

impl ToPolar for PolarValue {
    fn to_polar(self) -> PolarValue {
        self
    }
}

impl<T: ToPolar> ToPolar for Option<T> {
    fn to_polar(self) -> PolarValue {
        match self {
            Some(t) => t.to_polar(),
            None => PolarValue::new_from_instance(Option::<PolarValue>::None),
        }
    }
}

pub type PolarResultIter = Box<dyn Iterator<Item = Result<PolarValue, crate::OsoError>> + 'static>;

// Trait for the return value of class methods.
// This allows us to return polar values, as well as options and results of polar values.
pub trait ToPolarResults {
    fn to_polar_results(self) -> PolarResultIter;
}

impl<C: 'static + Sized + ToPolar> ToPolarResults for C {
    fn to_polar_results(self) -> PolarResultIter {
        Box::new(iter::once(Ok(self.to_polar())))
    }
}

impl<C, E> ToPolarResults for Result<C, E>
where
    C: ToPolarResults,
    E: std::error::Error + 'static + Send + Sync,
{
    fn to_polar_results(self) -> PolarResultIter {
        match self {
            Ok(result) => result.to_polar_results(),
            Err(e) => Box::new(iter::once(Err(crate::OsoError::ApplicationError {
                source: Box::new(e),
                type_name: None,
                attr: None,
            }))),
        }
    }
}

// NOTE: MISSING specialization... Want to have a variant for Result that
// is not over an error, but alas impossible???

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
    fn to_polar_results(self) -> PolarResultIter {
        Box::new(self.iter.flat_map(|i| i.to_polar_results())) as PolarResultIter
    }
}
