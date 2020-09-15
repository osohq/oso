use std::collections::HashMap;
use std::rc::Rc;

use polar_core::terms::{ExternalInstance, Numeric, Operator, Symbol, Term, Value};

use crate::Polar;

mod class;
mod class_method;
mod from_polar;
mod method;
mod to_polar;

pub use class::{Class, Instance};
pub use from_polar::FromPolar;
pub use to_polar::{PolarIter, ToPolar};

/// The meta class - the class of all classess (except itself)
#[derive(Clone, Default)]
pub struct Type;

fn type_class() -> Class {
    let class = Class::<Type>::with_default();
    class.erase_type()
}

/// Maintain mappings and caches for Rust classes & instances
pub struct Host {
    /// Reference to the inner `Polar` instance
    polar: Rc<Polar>,

    /// Map from names to `Class`s
    classes: HashMap<Symbol, Class>,

    /// Map of cached instances
    instances: HashMap<u64, class::Instance>,

    /// Map from type IDs, to class names
    /// This helps us go from a generic type `T` to the
    /// class name it is registered as
    class_names: HashMap<std::any::TypeId, Symbol>,
}

impl Host {
    pub fn new(polar: Rc<Polar>) -> Self {
        let mut host = Self {
            class_names: HashMap::new(),
            classes: HashMap::new(),
            instances: HashMap::new(),
            polar,
        };
        let type_class = type_class();
        let name = Symbol("Type".to_string());
        host.cache_class(type_class, name);
        host
    }

    pub fn type_class(&mut self) -> &mut Class {
        self.classes.get_mut(&Symbol("Type".to_string())).unwrap()
    }

    pub fn get_class(&self, name: &Symbol) -> Option<&Class> {
        self.classes.get(name)
    }

    pub fn get_class_from_type<C: 'static>(&self) -> Option<&Class> {
        self.class_names
            .get(&std::any::TypeId::of::<C>())
            .and_then(|name| self.get_class(name))
    }

    pub fn get_class_mut(&mut self, name: &Symbol) -> Option<&mut Class> {
        self.classes.get_mut(name)
    }

    /// Add the class to the host classes
    ///
    /// Returns an instance of `Type` for this class.
    pub fn cache_class(&mut self, class: Class, name: Symbol) -> String {
        self.class_names.insert(class.type_id, name.clone());
        self.classes.insert(name.clone(), class);
        name.0
    }

    pub fn get_instance(&self, id: u64) -> Option<&class::Instance> {
        self.instances.get(&id)
    }

    pub fn cache_instance(&mut self, instance: class::Instance, id: Option<u64>) -> u64 {
        let id = id.unwrap_or_else(|| self.polar.get_external_id());
        self.instances.insert(id, instance);
        id
    }

    pub fn make_instance(
        &mut self,
        name: &Symbol,
        fields: Vec<Term>,
        id: u64,
    ) -> crate::Result<()> {
        // @TODO: Handle the error if the class doesn't exist.
        let class = self.get_class(name).unwrap().clone();
        debug_assert!(self.instances.get(&id).is_none());
        let fields = fields; // TODO: use
        let instance = class.init(fields, self)?;
        self.cache_instance(instance, Some(id));
        Ok(())
    }

    pub fn unify(&self, left: u64, right: u64) -> bool {
        let _left = self.get_instance(left).unwrap();
        let _right = self.get_instance(right).unwrap();
        todo!("left == right")
    }

    pub fn isa(&self, term: Term, class_tag: &Symbol) -> bool {
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

    pub fn is_subspecializer(&self, id: u64, left_tag: &Symbol, right_tag: &Symbol) -> bool {
        let _instance = self.get_instance(id).unwrap();
        let _left = self.get_class(left_tag).unwrap();
        let _right = self.get_class(right_tag).unwrap();

        todo!("????")
    }

    pub fn operator(&self, _op: Operator, _args: [class::Instance; 2]) -> bool {
        todo!()
    }
}

/// Marker trait: implements "ToPolar" via a registered class
pub trait HostClass {}
