/// Translate between Polar and the host language (Rust).
use std::collections::HashMap;
use std::rc::Weak;

use polar::types::Symbol as Name;

#[derive(Clone)]
pub struct Class;

impl Class {
    pub fn name(&self) -> Name {
        todo!()
    }

    pub fn default_constructor(&self) -> Constructor {
        todo!()
    }
}
#[derive(Clone)]
pub struct Instance;
#[derive(Clone)]
pub struct Constructor;

impl Constructor {
    pub fn call(&self, fields: ()) -> Instance {
        todo!()
    }
}

/// Maintain mappings and caches for Python classes & instances
#[derive(Clone)]
pub struct Host {
    classes: HashMap<Name, Class>,
    constructors: HashMap<Name, Constructor>,
    instances: HashMap<u64, Instance>,
    polar: Weak<polar::Polar>,
}

impl Host {
    pub fn new(polar: Weak<polar::Polar>) -> Self {
        Self {
            classes: HashMap::new(),
            constructors: HashMap::new(),
            instances: HashMap::new(),
            polar,
        }
    }

    pub fn get_class(&self, name: &Name) -> Option<Class> {
        self.classes.get(name).cloned()
    }

    pub fn get_constructor(&self, name: &Name) -> Option<Constructor> {
        self.constructors.get(name).cloned()
    }

    pub fn cache_class(
        &mut self,
        class: Class,
        name: Option<Name>,
        constructor: Option<Constructor>,
    ) -> Name {
        let name = name.unwrap_or_else(|| class.name());
        let constructor = constructor.unwrap_or_else(|| class.default_constructor());
        self.classes.insert(name.clone(), class);
        self.constructors.insert(name.clone(), constructor);
        name
    }

    pub fn get_instance(&self, id: u64) -> Option<Instance> {
        self.instances.get(&id).cloned()
    }

    pub fn cache_instance(&mut self, instance: Instance, id: Option<u64>) -> u64 {
        let id = id.unwrap_or_else(|| self.polar.upgrade().unwrap().get_external_id());
        self.instances.insert(id, instance);
        id
    }

    pub fn make_instance(&mut self, name: &Name, fields: (), id: u64) -> Instance {
        let class = self.get_class(name).unwrap();
        let constructor = self.get_constructor(name).unwrap();
        debug_assert!(self.instances.get(&id).is_none());
        let instance = constructor.call(fields);
        self.cache_instance(instance.clone(), Some(id));
        instance
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

    pub fn operator(&self, op: polar::types::Operation, args: [Instance; 2]) -> bool {
        todo!()
    }

    pub fn to_polar(&mut self, value: &dyn ToPolar) -> Term {
        value.to_polar(self)
    }

    pub fn to_rust(&mut self, term: Term) -> impl std::any::Any {
        todo!()
    }
}

use polar::types::{Numeric, Term, Value};

trait ToPolar {
    fn to_polar_value(&self, host: &mut Host) -> Value;

    fn to_polar(&self, host: &mut Host) -> Term {
        Term::new_from_ffi(self.to_polar_value(host))
    }
}

impl ToPolar for bool {
    fn to_polar_value(&self, host: &mut Host) -> Value {
        Value::Boolean(*self)
    }
}

macro_rules! int_to_polar {
    ($i:ty) => {
        impl ToPolar for $i {
            fn to_polar_value(&self, host: &mut Host) -> Value {
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
            fn to_polar_value(&self, host: &mut Host) -> Value {
                Value::Number(Numeric::Float((*self).into()))
            }
        }
    };
}

float_to_polar!(f32);
float_to_polar!(f64);

impl ToPolar for String {
    fn to_polar_value(&self, host: &mut Host) -> Value {
        Value::String(self.clone())
    }
}

impl ToPolar for str {
    fn to_polar_value(&self, host: &mut Host) -> Value {
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
        Value::Dictionary(polar::types::Dictionary {
            fields: self
                .iter()
                .map(|(k, v)| (Name(k.to_string()), v.to_polar(host)))
                .collect(),
        })
    }
}

impl ToPolar for Value {
    fn to_polar_value(&self, _host: &mut Host) -> Value {
        self.clone()
    }
}

//     def to_python(self, value):
//         """Convert a Polar term to a Python object."""
//         value = value["value"]
//         tag = [*value][0]
//         if tag in ["String", "Boolean"]:
//             return value[tag]
//         elif tag == "Number":
//             return [*value[tag].values()][0]
//         elif tag == "List":
//             return [self.to_python(e) for e in value[tag]]
//         elif tag == "Dictionary":
//             return {k: self.to_python(v) for k, v in value[tag]["fields"].items()}
//         elif tag == "ExternalInstance":
//             return self.get_instance(value[tag]["instance_id"])
//         elif tag == "Call":
//             return Predicate(
//                 name=value[tag]["name"],
//                 args=[self.to_python(v) for v in value[tag]["args"]],
//             )
//         elif tag == "Variable":
//             return Variable(value[tag])

//         raise PolarRuntimeException(f"cannot convert {value} to Python")
