//! Trait and implementations of `ToPolar` for converting from
//! Rust types back to Polar types.

use polar_core::terms::*;

use std::collections::HashMap;

use super::Host;

pub trait ToPolar {
    fn to_polar_value(&self, host: &mut Host) -> Value;

    fn to_polar(&self, host: &mut Host) -> Term {
        Term::new_from_ffi(self.to_polar_value(host))
    }
}

impl ToPolar for bool {
    fn to_polar_value(&self, _host: &mut Host) -> Value {
        Value::Boolean(*self)
    }
}

macro_rules! int_to_polar {
    ($i:ty) => {
        impl ToPolar for $i {
            fn to_polar_value(&self, _host: &mut Host) -> Value {
                Value::Number(Numeric::Integer((*self).into()))
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
            fn to_polar_value(&self, _host: &mut Host) -> Value {
                Value::Number(Numeric::Float((*self).into()))
            }
        }
    };
}

float_to_polar!(f32);
float_to_polar!(f64);

impl ToPolar for String {
    fn to_polar_value(&self, _host: &mut Host) -> Value {
        Value::String(self.clone())
    }
}

impl ToPolar for &'static str {
    fn to_polar_value(&self, _host: &mut Host) -> Value {
        Value::String(self.to_string())
    }
}

impl ToPolar for str {
    fn to_polar_value(&self, _host: &mut Host) -> Value {
        Value::String(self.to_owned())
    }
}

impl<T: ToPolar> ToPolar for Vec<T> {
    fn to_polar_value(&self, host: &mut Host) -> Value {
        Value::List(self.iter().map(|v| v.to_polar(host)).collect())
    }
}

impl<T: ToPolar> ToPolar for HashMap<String, T> {
    fn to_polar_value(&self, host: &mut Host) -> Value {
        Value::Dictionary(Dictionary {
            fields: self
                .iter()
                .map(|(k, v)| (Symbol(k.to_string()), v.to_polar(host)))
                .collect(),
        })
    }
}

pub struct PolarIter<I>(pub I);

impl<I: Clone + Iterator<Item = T>, T: ToPolar> ToPolar for PolarIter<I> {
    fn to_polar_value(&self, host: &mut Host) -> Value {
        Value::List(self.0.clone().map(|v| v.to_polar(host)).collect())
    }
}

impl ToPolar for Value {
    fn to_polar_value(&self, _host: &mut Host) -> Value {
        self.clone()
    }
}

impl ToPolar for Box<dyn ToPolar> {
    fn to_polar_value(&self, host: &mut Host) -> Value {
        self.as_ref().to_polar_value(host)
    }
}

impl ToPolar for crate::Class {
    fn to_polar_value(&self, host: &mut Host) -> Value {
        let type_class = host.type_class();
        for method_name in self.class_methods.keys() {
            type_class
                .instance_methods
                .entry(method_name.clone())
                .or_insert_with(|| {
                    super::class_method::InstanceMethod::from_class_method(method_name.clone())
                });
        }
        let repr = format!("type<{}>", self.name);
        let instance = type_class.cast_to_instance(self.clone());
        let instance = host.cache_instance(instance, None);
        Value::ExternalInstance(ExternalInstance {
            constructor: None,
            repr: Some(repr),
            instance_id: instance,
        })
    }
}

impl<C: 'static + Clone + super::HostClass> ToPolar for C {
    fn to_polar_value(&self, host: &mut Host) -> Value {
        let class = host
            .get_class_from_type::<C>()
            .expect("Class not registered");
        let instance = class.cast_to_instance(self.clone());
        let instance = host.cache_instance(instance, None);
        Value::ExternalInstance(ExternalInstance {
            constructor: None,
            repr: None,
            instance_id: instance,
        })
    }
}

use std::iter;

// Trait for the return value of class methods.
// This allows us to return polar values, as well as options and results of polar values.
// Iterators and futures too once we get to them.
pub trait ToPolarIter {
    fn to_polar_iter(&self) -> Box<dyn Iterator<Item = Result<Box<dyn ToPolar>, crate::OsoError>>>;
}

impl<C: 'static + Sized + Clone + ToPolar> ToPolarIter for C {
    fn to_polar_iter(&self) -> Box<dyn Iterator<Item = Result<Box<dyn ToPolar>, crate::OsoError>>> {
        Box::new(iter::once(Ok(Box::new(self.clone()) as Box<dyn ToPolar>)))
    }
}

impl<C: ToPolarIter, E: ToString> ToPolarIter for Result<C, E> {
    fn to_polar_iter(&self) -> Box<dyn Iterator<Item = Result<Box<dyn ToPolar>, crate::OsoError>>> {
        match self {
            Ok(result) => result.to_polar_iter(),
            Err(e) => Box::new(iter::once(Err(crate::OsoError::Custom {
                message: e.to_string(),
            }))),
        }
    }
}

impl<C: ToPolarIter> ToPolarIter for Option<C> {
    fn to_polar_iter(&self) -> Box<dyn Iterator<Item = Result<Box<dyn ToPolar>, crate::OsoError>>> {
        self.as_ref().map_or_else(
            || {
                Box::new(std::iter::empty())
                    as Box<dyn Iterator<Item = Result<Box<dyn ToPolar>, crate::OsoError>>>
            },
            |e| e.to_polar_iter(),
        )
    }
}
