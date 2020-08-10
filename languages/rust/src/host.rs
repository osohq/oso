/// Translate between Polar and the host language (Rust).
use std::collections::{BTreeMap, HashMap};
use std::rc::Weak;
use std::sync::Arc;

use polar_core::types::Symbol as Name;
use polar_core::types::{Numeric, Term, Value};

#[derive(Clone)]
pub struct Class {
    name: String,
    constructor: Arc<dyn Fn() -> Arc<dyn std::any::Any>>,
    instance_methods: InstanceMethods,
    class_methods: ClassMethods,
}

impl Class {
    pub fn new<T: std::default::Default + 'static>() -> Self {
        Self {
            name: std::any::type_name::<Self>().to_string(),
            constructor: Arc::new(|| Arc::new(T::default())),
            instance_methods: InstanceMethods::new(),
            class_methods: ClassMethods::new(),
        }
    }

    pub fn with_constructor<T: 'static, F: 'static + Fn() -> T>(constructor: F) -> Self {
        Self {
            name: std::any::type_name::<Self>().to_string(),
            constructor: Arc::new(move || Arc::new(constructor())),
            instance_methods: InstanceMethods::new(),
            class_methods: ClassMethods::new(),
        }
    }

    pub fn add_method<T: 'static, R: 'static + ToPolar, F: 'static + Fn(&T) -> R>(
        &mut self,
        name: &str,
        f: F,
    ) {
        self.instance_methods.insert(
            Name(name.to_string()),
            Arc::new(
                move |self_arg: &Instance, args: Vec<polar_core::types::Term>| {
                    assert!(args.is_empty());
                    let self_arg = self_arg
                        .instance
                        .downcast_ref::<T>()
                        .expect(&format!("not a {}!", std::any::type_name::<T>()));
                    let result = f(&self_arg);
                    Arc::new(result) as Arc<dyn ToPolar>
                },
            ) as InstanceMethod,
        );
    }

    pub fn add_class_method<T: 'static, R: 'static + ToPolar, F: 'static + Fn() -> R>(
        &mut self,
        name: &str,
        f: F,
    ) {
        self.class_methods.insert(
            Name(name.to_string()),
            Arc::new(move |class: &Class, args: Vec<polar_core::types::Term>| {
                assert!(args.is_empty());
                let result = f();
                Arc::new(result) as Arc<dyn ToPolar>
            }) as ClassMethod,
        );
    }

    pub fn register(
        self,
        name: Option<String>,
        polar: &mut crate::polar::Polar,
    ) -> anyhow::Result<()> {
        polar.register_class(self, name)?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct Instance {
    pub instance: Arc<dyn std::any::Any>,
    pub methods: Arc<InstanceMethods>,
}

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

    pub fn make_instance(&mut self, name: &Name, fields: BTreeMap<Name, Term>, id: u64) {
        let class = self.get_class(name).unwrap();
        debug_assert!(self.instances.get(&id).is_none());
        let _fields = fields; // TODO: use
        let instance = (class.constructor)();
        let instance = Instance {
            instance,
            methods: Arc::new(class.instance_methods.clone()),
        };
        self.cache_instance(instance, Some(id));
    }

    pub fn unify(&self, left: u64, right: u64) -> bool {
        let left = self.get_instance(left).unwrap();
        let right = self.get_instance(right).unwrap();
        todo!("left == right")
    }

    pub fn isa(&self, id: u64, class_tag: &Name) -> bool {
        let instance = self.get_instance(id).unwrap();
        let class = self.get_class(class_tag).unwrap();
        todo!("isinstance(instance, class)")
    }

    pub fn is_subspecializer(&self, id: u64, left_tag: &Name, right_tag: &Name) -> bool {
        let instance = self.get_instance(id).unwrap();
        let left = self.get_class(left_tag).unwrap();
        let right = self.get_class(right_tag).unwrap();

        todo!("????")
    }

    pub fn operator(&self, op: polar_core::types::Operation, args: [Instance; 2]) -> bool {
        todo!()
    }

    pub fn to_polar(&mut self, value: &dyn ToPolar) -> Term {
        value.to_polar(self)
    }

    pub fn to_rust(&mut self, term: Term) -> impl std::any::Any {
        todo!()
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

pub type InstanceMethod = Arc<dyn Fn(&Instance, Vec<Term>) -> Arc<dyn ToPolar>>;
pub type InstanceMethods = HashMap<Name, InstanceMethod>;

pub type ClassMethod = Arc<dyn Fn(&Class, Vec<Term>) -> Arc<dyn ToPolar>>;
pub type ClassMethods = HashMap<Name, ClassMethod>;
