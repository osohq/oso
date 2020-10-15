use std::collections::HashMap;
use std::sync::Arc;

use polar_core::terms::{self, ExternalInstance, Numeric, Operator, Symbol, Term};

use crate::errors::OsoError;
use crate::Polar;

mod class;
mod class_method;
mod from_polar;
mod method;
mod to_polar;
mod value;

pub use value::*;

pub use class::{Class, ClassBuilder, Instance};
pub use from_polar::{FromPolar, FromPolarList};
pub use to_polar::{PolarResultIter, ToPolar, ToPolarList, ToPolarResults};

impl ToPolar for crate::Class {}
fn metaclass() -> Class {
    Class::builder::<Class>().name("oso::host::Class").build()
}

/// Maintain mappings and caches for Rust classes & instances
#[derive(Clone)]
pub struct Host {
    /// Reference to the inner `Polar` instance
    polar: Arc<Polar>,

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
    pub fn new(polar: Arc<Polar>) -> Self {
        let mut host = Self {
            class_names: HashMap::new(),
            classes: HashMap::new(),
            instances: HashMap::new(),
            polar,
        };
        let type_class = metaclass();
        let name = Symbol(type_class.name.clone());
        host.cache_class(type_class, name)
            .expect("could not register the metaclass");
        host
    }

    pub fn get_class(&self, name: &Symbol) -> crate::Result<&Class> {
        self.classes
            .get(name)
            .ok_or_else(|| OsoError::MissingClassError {
                name: name.0.clone(),
            })
    }

    pub fn get_class_by_type_id(&self, id: std::any::TypeId) -> crate::Result<&Class> {
        self.class_names
            .get(&id)
            .ok_or_else(|| OsoError::MissingClassError {
                name: format!("TypeId: {:?}", id),
            })
            .and_then(|name| self.get_class(name))
    }

    pub fn get_class_from_type<C: 'static>(&self) -> crate::Result<&Class> {
        self.get_class_by_type_id(std::any::TypeId::of::<C>())
    }

    pub fn get_class_mut(&mut self, name: &Symbol) -> crate::Result<&mut Class> {
        self.classes
            .get_mut(name)
            .ok_or_else(|| OsoError::MissingClassError {
                name: name.0.clone(),
            })
    }

    /// Add the class to the host classes
    ///
    /// Returns an instance of `Type` for this class.
    pub fn cache_class(&mut self, class: Class, name: Symbol) -> crate::Result<String> {
        if self.classes.contains_key(&name) {
            return Err(OsoError::DuplicateClassError { name: name.0 });
        }

        self.class_names.insert(class.type_id, name.clone());
        self.classes.insert(name.clone(), class);
        Ok(name.0)
    }

    pub fn get_instance(&self, id: u64) -> crate::Result<&class::Instance> {
        tracing::trace!("instances: {:?}", self.instances.keys().collect::<Vec<_>>());
        self.instances
            .get(&id)
            .ok_or_else(|| OsoError::MissingInstanceError)
    }

    pub fn cache_instance(&mut self, instance: class::Instance, id: Option<u64>) -> u64 {
        let id = id.unwrap_or_else(|| self.polar.get_external_id());
        tracing::trace!(
            "insert instance {:?} {:?}, instances: {:?}",
            id,
            instance,
            self.instances.keys().collect::<Vec<_>>()
        );
        self.instances.insert(id, instance);
        id
    }

    pub fn make_instance(
        &mut self,
        name: &Symbol,
        fields: Vec<Term>,
        id: u64,
    ) -> crate::Result<()> {
        let class = self.get_class(name)?.clone();
        debug_assert!(self.instances.get(&id).is_none());
        let fields = fields;
        let instance = class.init(fields, self)?;
        self.cache_instance(instance, Some(id));
        Ok(())
    }

    pub fn unify(&self, left: u64, right: u64) -> crate::Result<bool> {
        tracing::trace!("unify {:?}, {:?}", left, right);

        let left = self.get_instance(left).unwrap();
        let right = self.get_instance(right).unwrap();
        left.equals(right, &self)
    }

    pub fn isa(&self, term: Term, class_tag: &Symbol) -> crate::Result<bool> {
        let name = &class_tag.0;
        let res = match term.value() {
            terms::Value::ExternalInstance(ExternalInstance { instance_id, .. }) => {
                let class = self.get_class(class_tag)?;
                let instance = self.get_instance(*instance_id)?;
                instance.instance_of(class)
            }
            terms::Value::Boolean(_) => name == "Boolean",
            terms::Value::Dictionary(_) => name == "Dictionary",
            terms::Value::List(_) => name == "List",
            terms::Value::Number(n) => {
                name == "Number"
                    || match n {
                        Numeric::Integer(_) => name == "Integer",
                        Numeric::Float(_) => name == "Float",
                    }
            }
            terms::Value::String(_) => name == "String",
            _ => false,
        };
        Ok(res)
    }

    pub fn is_subspecializer(&self, _id: u64, _left_tag: &Symbol, _right_tag: &Symbol) -> bool {
        // Rust has no notion of inheritance, so there are no subspecializers.
        false
    }

    pub fn operator(&self, _op: Operator, _args: [class::Instance; 2]) -> crate::Result<bool> {
        // Operators are not supported
        // TODO (dhatch): Implement.
        Err(OsoError::UnimplementedOperation {
            operation: String::from("comparison operators"),
        })
    }
}
