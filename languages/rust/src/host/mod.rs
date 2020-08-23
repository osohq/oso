//! Translate between Polar and the host language (Rust).

use std::any::Any;
use std::collections::HashMap;
use std::rc::Weak;
use std::sync::Arc;

use polar_core::types::Symbol as Name;
use polar_core::types::{ExternalInstance, Numeric, Operator, Term, Value};

mod class;
pub use class::*;

#[derive(Clone, Default)]
pub struct Type;

pub fn type_class() -> Class {
    let class = Class::<Type>::with_default();
    class.erase_type()
}

/// Maintain mappings and caches for Python classes & instances
pub struct Host {
    polar: Weak<crate::PolarCore>,
    classes: HashMap<Name, Class>,
    instances: HashMap<u64, Instance>,
    class_names: HashMap<std::any::TypeId, Name>,
}

impl Host {
    pub fn new(polar: Weak<crate::PolarCore>) -> Self {
        let mut host = Self {
            class_names: HashMap::new(),
            classes: HashMap::new(),
            instances: HashMap::new(),
            polar,
        };
        let type_class = type_class();
        let name = Name("Type".to_string());
        host.class_names.insert(type_class.type_id, name.clone());
        host.classes.insert(name.clone(), type_class);
        host
    }

    pub fn type_class(&mut self) -> &mut Class {
        self.classes.get_mut(&Name("Type".to_string())).unwrap()
    }

    pub fn get_class(&self, name: &Name) -> Option<&Class> {
        self.classes.get(name)
    }

    pub fn get_class_from_type<C: 'static>(&self) -> Option<&Class> {
        self.class_names
            .get(&std::any::TypeId::of::<C>())
            .and_then(|name| self.get_class(name))
    }

    pub fn get_class_mut(&mut self, name: &Name) -> Option<&mut Class> {
        self.classes.get_mut(name)
    }

    pub fn cache_class(&mut self, class: Class, name: Name) -> Term {
        self.class_names.insert(class.type_id, name.clone());
        self.classes.insert(name.clone(), class.clone());

        let type_class = self.type_class();
        for method_name in class.class_methods.keys() {
            type_class
                .instance_methods
                .entry(method_name.clone())
                .or_insert_with(|| {
                    crate::host::InstanceMethod::from_class_method(method_name.clone())
                });
        }
        let repr = format!("type<{}>", class.name);
        let instance = type_class.cast_to_instance(class.clone());
        let instance = self.cache_instance(instance, None);
        let class_term = Term::new_from_ffi(Value::ExternalInstance(ExternalInstance {
            constructor: None,
            repr: Some(repr),
            instance_id: instance,
        }));

        class_term
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
        let class = self.get_class(name).unwrap().clone();
        debug_assert!(self.instances.get(&id).is_none());
        let fields = fields; // TODO: use
        let instance = class.new(fields, self);
        self.cache_instance(instance, Some(id));
    }

    pub fn unify(&self, left: u64, right: u64) -> bool {
        let _left = self.get_instance(left).unwrap();
        let _right = self.get_instance(right).unwrap();
        todo!("left == right")
    }

    pub fn isa(&self, term: Term, class_tag: &Name) -> bool {
        let name = &class_tag.0;
        match term.value() {
            Value::ExternalInstance(ExternalInstance { instance_id, .. }) => {
                let class = self.get_class(class_tag).unwrap();
                let instance = self.get_instance(*instance_id).unwrap();
                class.is_instance(instance)
            }
            Value::Boolean(_) => name == "Boolean",
            Value::Dictionary(_) => name == "Dictionary",
            Value::List(_) => name == "List",
            Value::Number(n) => {
                name == "Number"
                    || match n {
                        Numeric::Integer(_) => name == "Integer",
                        Numeric::Float(_) => name == "Float",
                    }
            }
            Value::String(_) => name == "String",
            _ => false,
        }
    }

    pub fn is_subspecializer(&self, id: u64, left_tag: &Name, right_tag: &Name) -> bool {
        let _instance = self.get_instance(id).unwrap();
        let _left = self.get_class(left_tag).unwrap();
        let _right = self.get_class(right_tag).unwrap();

        todo!("????")
    }

    pub fn operator(&self, _op: Operator, _args: [Instance; 2]) -> bool {
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
                .get_class_from_type::<HashMap<Name, Term>>()
                .unwrap()
                .cast_to_instance(d.fields),
            Value::ExternalInstance(ExternalInstance { instance_id, .. }) => host
                .get_instance(instance_id)
                .expect("instance not found")
                .clone(),
            v => {
                tracing::warn!(value = ?v, "invalid conversion attempted");
                return None;
            }
        };
        Some(instance)
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

/// Marker trait: implements "ToPolar" via a registered class
pub trait HostClass {}

impl<C: 'static + Clone + HostClass> FromPolar for C {
    fn from_polar(term: &Term, host: &mut Host) -> Option<Self> {
        match term.value() {
            Value::ExternalInstance(ExternalInstance { instance_id, .. }) => {
                let instance = host.get_instance(*instance_id)?;
                instance.instance.downcast_ref::<C>().cloned()
            }
            _ => None,
        }
    }
}

impl<C: 'static + Clone + HostClass> ToPolar for C {
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
