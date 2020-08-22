//! Translate between Polar and the host language (Rust).

use std::any::Any;
use std::collections::HashMap;
use std::rc::Weak;
use std::sync::Arc;

use polar_core::types::Symbol as Name;
use polar_core::types::{Numeric, Term, Value};

#[path = "class/mod.rs"]
mod class;
pub use class::*;

/// Maintain mappings and caches for Python classes & instances
pub struct Host {
    classes: HashMap<Name, Class>,
    instances: HashMap<u64, Instance>,
    polar: Weak<crate::PolarCore>,
}

impl Host {
    pub fn new(polar: Weak<crate::PolarCore>) -> Self {
        Self {
            classes: HashMap::new(),
            instances: HashMap::new(),
            polar,
        }
    }

    pub fn get_class(&self, name: &Name) -> Option<&Class> {
        self.classes.get(name)
    }

    pub fn get_class_mut(&mut self, name: &Name) -> Option<&mut Class> {
        self.classes.get_mut(name)
    }

    pub fn cache_class(&mut self, class: Class, name: Option<Name>) -> Name {
        let name = name.unwrap_or_else(|| Name(class.name.clone()));
        self.classes.insert(name.clone(), class);
        name
    }

    pub fn get_instance(&self, id: u64) -> Option<&Instance> {
        self.instances.get(&id)
    }

    pub fn cache_instance(&mut self, instance: Instance, id: Option<u64>) -> u64 {
        let id = id.unwrap_or_else(|| self.polar.upgrade().unwrap().get_external_id());
        self.instances.insert(id, instance);
        id
    }

    pub fn make_instance(&mut self, name: &Name, fields: Vec<Term>, id: u64) {
        let Class {
            constructor,
            attributes,
            instance_methods,
            ..
        } = self.get_class(name).unwrap().clone();
        debug_assert!(self.instances.get(&id).is_none());
        let fields = fields; // TODO: use
        let instance = constructor.invoke(fields, self);
        let instance = Instance {
            instance,
            attributes: Arc::new(attributes),
            methods: Arc::new(instance_methods),
        };
        self.cache_instance(instance, Some(id));
    }

    pub fn unify(&self, left: u64, right: u64) -> bool {
        let _left = self.get_instance(left).unwrap();
        let _right = self.get_instance(right).unwrap();
        todo!("left == right")
    }

    pub fn isa(&self, id: u64, class_tag: &Name) -> bool {
        let instance = self.get_instance(id).unwrap();
        let class = self.get_class(class_tag).unwrap();
        class.isinstance(instance)
    }

    pub fn is_subspecializer(&self, id: u64, left_tag: &Name, right_tag: &Name) -> bool {
        let _instance = self.get_instance(id).unwrap();
        let _left = self.get_class(left_tag).unwrap();
        let _right = self.get_class(right_tag).unwrap();

        todo!("????")
    }

    pub fn operator(&self, _op: polar_core::types::Operator, _args: [u64; 2]) -> bool {
        todo!()
    }

    pub fn to_polar(&mut self, value: &dyn ToPolar) -> Term {
        value.to_polar(self)
    }
}

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
        Value::Dictionary(polar_core::types::Dictionary {
            fields: self
                .iter()
                .map(|(k, v)| (Name(k.to_string()), v.to_polar(host)))
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

pub trait FromPolar: Sized {
    fn from_polar(term: &Term, host: &mut Host) -> Option<Self>;

    fn from_polar_list(terms: &[Term], host: &mut Host) -> Option<Self> {
        assert_eq!(terms.len(), 1);
        Self::from_polar(&terms[0], host)
    }
}

impl FromPolar for bool {
    fn from_polar(term: &Term, _host: &mut Host) -> Option<Self> {
        if let Value::Boolean(b) = term.value() {
            Some(*b)
        } else {
            None
        }
    }
}

use std::convert::TryFrom;

macro_rules! polar_to_int {
    ($i:ty) => {
        impl FromPolar for $i {
            fn from_polar(term: &Term, _host: &mut Host) -> Option<Self> {
                if let Value::Number(Numeric::Integer(i)) = term.value() {
                    <$i>::try_from(*i).ok()
                } else {
                    None
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
    fn from_polar(term: &Term, _host: &mut Host) -> Option<Self> {
        if let Value::Number(Numeric::Float(f)) = term.value() {
            Some(*f)
        } else {
            None
        }
    }
}

impl FromPolar for String {
    fn from_polar(term: &Term, _host: &mut Host) -> Option<Self> {
        if let Value::String(s) = term.value() {
            Some(s.to_string())
        } else {
            None
        }
    }
}

impl<T: FromPolar> FromPolar for Vec<T> {
    fn from_polar(term: &Term, host: &mut Host) -> Option<Self> {
        if let Value::List(l) = term.value() {
            l.iter().map(|t| T::from_polar(t, host)).collect()
        } else {
            None
        }
    }

    fn from_polar_list(terms: &[Term], host: &mut Host) -> Option<Self> {
        terms.iter().map(|t| T::from_polar(t, host)).collect()
    }
}

impl<T: FromPolar> FromPolar for HashMap<String, T> {
    fn from_polar(term: &Term, host: &mut Host) -> Option<Self> {
        if let Value::Dictionary(dict) = term.value() {
            dict.fields
                .iter()
                .map(|(k, v)| T::from_polar(v, host).map(|v| (k.0.clone(), v)))
                .collect()
        } else {
            None
        }
    }
}

impl FromPolar for Value {
    fn from_polar(term: &Term, _host: &mut Host) -> Option<Self> {
        Some(term.value().clone())
    }
}

impl FromPolar for Instance {
    fn from_polar(term: &Term, host: &mut Host) -> Option<Self> {
        if let Value::ExternalInstance(polar_core::types::ExternalInstance {
            instance_id, ..
        }) = term.value()
        {
            host.get_instance(*instance_id).cloned()
        } else {
            None
        }
    }
}

impl FromPolar for () {
    fn from_polar(term: &Term, _host: &mut Host) -> Option<Self> {
        if let Value::List(l) = term.value() {
            if l.is_empty() {
                Some(())
            } else {
                None
            }
        } else {
            None
        }
    }

    fn from_polar_list(terms: &[Term], _host: &mut Host) -> Option<Self> {
        if terms.is_empty() {
            Some(())
        } else {
            None
        }
    }
}

impl<A> FromPolar for (A,)
where
    A: FromPolar,
{
    fn from_polar(_term: &Term, _host: &mut Host) -> Option<Self> {
        None
    }

    fn from_polar_list(terms: &[Term], host: &mut Host) -> Option<Self> {
        if terms.len() == 1 {
            A::from_polar(&terms[0], host).map(|a| (a,))
        } else {
            None
        }
    }
}

impl<A, B> FromPolar for (A, B)
where
    A: FromPolar,
    B: FromPolar,
{
    fn from_polar(_term: &Term, _host: &mut Host) -> Option<Self> {
        None
    }

    fn from_polar_list(terms: &[Term], host: &mut Host) -> Option<Self> {
        if terms.len() == 2 {
            let a = A::from_polar(&terms[0], host)?;
            let b = B::from_polar(&terms[1], host)?;
            Some((a, b))
        } else {
            None
        }
    }
}
