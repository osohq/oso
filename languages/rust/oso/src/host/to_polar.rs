//! Trait and implementations of `ToPolar` for converting from
//! Rust types back to Polar types.

use impl_trait_for_tuples::*;

use std::collections::HashMap;

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

pub trait ToPolarResult {
    fn to_polar_result(self) -> crate::Result<PolarValue>;
}

impl<R: ToPolar> ToPolarResult for R {
    fn to_polar_result(self) -> crate::Result<PolarValue> {
        Ok(self.to_polar())
    }
}

impl<E: std::error::Error + Send + Sync + 'static, R: ToPolar> ToPolarResult for Result<R, E> {
    fn to_polar_result(self) -> crate::Result<PolarValue> {
        self.map(|r| r.to_polar())
            .map_err(|e| crate::OsoError::ApplicationError {
                source: Box::new(e),
                attr: None,
                type_name: None,
            })
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

pub struct PolarIterator(pub Box<dyn PolarResultIter>);

impl PolarIterator {
    pub fn new<I: PolarResultIter + 'static>(iter: I) -> Self {
        Self(Box::new(iter))
    }

    pub fn next(&mut self) -> Option<crate::Result<PolarValue>> {
        self.0.next()
    }
}

impl Clone for PolarIterator {
    fn clone(&self) -> Self {
        Self(self.0.box_clone())
    }
}
impl crate::PolarClass for PolarIterator {}

pub trait PolarResultIter: Send + Sync {
    fn box_clone(&self) -> Box<dyn PolarResultIter>;
    fn next(&mut self) -> Option<crate::Result<PolarValue>>;
}

impl<I, V> PolarResultIter for I
where
    I: Iterator<Item = V> + Clone + Send + Sync + 'static,
    V: ToPolarResult,
{
    fn box_clone(&self) -> Box<dyn PolarResultIter> {
        Box::new(self.clone())
    }

    fn next(&mut self) -> Option<crate::Result<PolarValue>> {
        Iterator::next(self).map(|v| v.to_polar_result())
    }
}
