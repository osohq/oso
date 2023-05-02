#![allow(clippy::many_single_char_names, clippy::type_complexity)]
//! Trait and implementations of `FromPolar` for converting from
//! Polar types back to Rust types.

use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque};
use std::hash::Hash;

use impl_trait_for_tuples::*;

use super::class::Instance;
use super::PolarValue;
use crate::errors::TypeError;
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
/// `FromPolar` requires `Clone` because we can only
/// get a borrowed value back from oso. In the future, this could
/// be updated to return borrowed data instead.
///
/// The default implementation for `PolarClass`
/// also requires types to be `Send + Sync`, since it
/// is possible to store a `FromPolar` value on an `Oso` instance
/// which can be shared between threads
pub trait FromPolar: Clone {
    fn from_polar(val: PolarValue) -> crate::Result<Self>;
}

impl FromPolar for PolarValue {
    fn from_polar(val: PolarValue) -> crate::Result<Self> {
        Ok(val)
    }
}

macro_rules! polar_to_int {
    ($i:ty) => {
        impl FromPolar for $i {
            fn from_polar(val: PolarValue) -> crate::Result<Self> {
                if let PolarValue::Integer(i) = val {
                    <$i>::try_from(i).map_err(|_| crate::OsoError::FromPolar)
                } else {
                    Err(TypeError::expected("Integer").user())
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

impl<T> FromPolar for T
where
    T: 'static + Clone + PolarClass,
{
    fn from_polar(val: PolarValue) -> crate::Result<Self> {
        if let PolarValue::Instance(instance) = val {
            Ok(instance.downcast::<T>(None).map_err(|e| e.user())?.clone())
        } else {
            Err(
                TypeError::expected(format!("Instance of {}", std::any::type_name::<T>()))
                    .got(val.type_name().to_string())
                    .user(),
            )
        }
    }
}

impl FromPolar for f64 {
    fn from_polar(val: PolarValue) -> crate::Result<Self> {
        if let PolarValue::Float(f) = val {
            Ok(f)
        } else {
            Err(TypeError::expected("Float").user())
        }
    }
}

impl FromPolar for String {
    fn from_polar(val: PolarValue) -> crate::Result<Self> {
        if let PolarValue::String(s) = val {
            Ok(s)
        } else {
            Err(TypeError::expected("String").user())
        }
    }
}

impl FromPolar for bool {
    fn from_polar(val: PolarValue) -> crate::Result<Self> {
        if let PolarValue::Boolean(b) = val {
            Ok(b)
        } else {
            Err(TypeError::expected("Boolean").user())
        }
    }
}

impl<T: FromPolar> FromPolar for HashMap<String, T> {
    fn from_polar(val: PolarValue) -> crate::Result<Self> {
        if let PolarValue::Map(map) = val {
            let mut result = HashMap::new();
            for (k, v) in map {
                let val = T::from_polar(v)?;
                result.insert(k, val);
            }
            Ok(result)
        } else {
            Err(TypeError::expected("Map").user())
        }
    }
}

impl<T: FromPolar> FromPolar for BTreeMap<String, T> {
    fn from_polar(val: PolarValue) -> crate::Result<Self> {
        if let PolarValue::Map(map) = val {
            let mut result = BTreeMap::new();
            for (k, v) in map {
                let val = T::from_polar(v)?;
                result.insert(k, val);
            }
            Ok(result)
        } else {
            Err(TypeError::expected("Map").user())
        }
    }
}

impl<T: FromPolar> FromPolar for Vec<T> {
    fn from_polar(val: PolarValue) -> crate::Result<Self> {
        if let PolarValue::List(l) = val {
            let mut result = vec![];
            for v in l {
                result.push(T::from_polar(v)?);
            }
            Ok(result)
        } else {
            Err(TypeError::expected("List").user())
        }
    }
}

impl<T: FromPolar> FromPolar for LinkedList<T> {
    fn from_polar(val: PolarValue) -> crate::Result<Self> {
        if let PolarValue::List(l) = val {
            let mut result = LinkedList::new();
            for v in l {
                result.push_back(T::from_polar(v)?);
            }
            Ok(result)
        } else {
            Err(TypeError::expected("List").user())
        }
    }
}

impl<T: FromPolar> FromPolar for VecDeque<T> {
    fn from_polar(val: PolarValue) -> crate::Result<Self> {
        if let PolarValue::List(l) = val {
            let mut result = VecDeque::new();
            for v in l {
                result.push_back(T::from_polar(v)?);
            }
            Ok(result)
        } else {
            Err(TypeError::expected("List").user())
        }
    }
}

impl<T: Eq + Hash + FromPolar> FromPolar for HashSet<T> {
    fn from_polar(val: PolarValue) -> crate::Result<Self> {
        if let PolarValue::List(l) = val {
            let mut result = HashSet::new();
            for v in l {
                result.insert(T::from_polar(v)?);
            }
            Ok(result)
        } else {
            Err(TypeError::expected("List").user())
        }
    }
}

impl<T: Eq + Ord + FromPolar> FromPolar for BTreeSet<T> {
    fn from_polar(val: PolarValue) -> crate::Result<Self> {
        if let PolarValue::List(l) = val {
            let mut result = BTreeSet::new();
            for v in l {
                result.insert(T::from_polar(v)?);
            }
            Ok(result)
        } else {
            Err(TypeError::expected("List").user())
        }
    }
}

impl<T: Ord + FromPolar> FromPolar for BinaryHeap<T> {
    fn from_polar(val: PolarValue) -> crate::Result<Self> {
        if let PolarValue::List(l) = val {
            let mut result = BinaryHeap::new();
            for v in l {
                result.push(T::from_polar(v)?);
            }
            Ok(result)
        } else {
            Err(TypeError::expected("List").user())
        }
    }
}

impl<T: FromPolar> FromPolar for Option<T> {
    fn from_polar(val: PolarValue) -> crate::Result<Self> {
        // if the value is a Option<PolarValue>, convert from PolarValue
        if let PolarValue::Instance(ref instance) = &val {
            if let Ok(opt) = instance.downcast::<Option<PolarValue>>(None) {
                return opt.clone().map(T::from_polar).transpose();
            }
        }
        T::from_polar(val).map(Some)
    }
}

// well, you can't do this
// impl<U: FromPolar> TryFrom<U> for PolarValue {
//     type Error = crate::OsoError;

//     fn try_from(v: PolarValue) -> Result<Self, Self::Error> {
//         U::from_polar(v)
//     }
// }

// so I have to do this
macro_rules! try_from_polar {
    ($i:ty) => {
        impl TryFrom<PolarValue> for $i {
            type Error = crate::OsoError;

            fn try_from(v: PolarValue) -> Result<Self, Self::Error> {
                Self::from_polar(v)
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

impl<T: FromPolar> TryFrom<PolarValue> for HashMap<String, T> {
    type Error = crate::OsoError;

    fn try_from(v: PolarValue) -> Result<Self, Self::Error> {
        Self::from_polar(v)
    }
}

impl<T: FromPolar> TryFrom<PolarValue> for Vec<T> {
    type Error = crate::OsoError;

    fn try_from(v: PolarValue) -> Result<Self, Self::Error> {
        Self::from_polar(v)
    }
}

mod private {
    /// Prevents implementations of `FromPolarList` outside of this crate
    pub trait Sealed {}
}

pub trait FromPolarList: private::Sealed {
    fn from_polar_list(values: &[PolarValue]) -> crate::Result<Self>
    where
        Self: Sized;
}

impl FromPolar for Instance {
    fn from_polar(value: PolarValue) -> crate::Result<Self> {
        // We need to handle converting all value variants to an
        // instance so that we can use the `Class` mechanism to
        // handle methods on them
        let instance = match value {
            PolarValue::Boolean(b) => Instance::new(b),
            PolarValue::Integer(i) => Instance::new(i),
            PolarValue::Float(f) => Instance::new(f),
            PolarValue::List(v) => Instance::new(v),
            PolarValue::String(s) => Instance::new(s),
            PolarValue::Map(d) => Instance::new(d),
            PolarValue::Instance(instance) => instance,
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
    fn from_polar_list(values: &[PolarValue]) -> crate::Result<Self> {
        let mut iter = values.iter();
        let result = Ok((for_tuples!(
            #( Tuple::from_polar(iter.next().ok_or(
                // TODO better error type
                crate::OsoError::FromPolar
            )?.clone())? ),*
        )));

        if iter.len() > 0 {
            // TODO (dhatch): Debug this!!!
            tracing::warn!("Remaining items in iterator after conversion.");
            for item in iter {
                tracing::trace!("Remaining item {:?}", item);
            }

            return Err(crate::OsoError::FromPolar);
        }

        result
    }
}

#[impl_for_tuples(16)]
#[tuple_types_custom_trait_bound(FromPolar)]
impl private::Sealed for Tuple {}
