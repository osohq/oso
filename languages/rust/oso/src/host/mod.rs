use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::errors::OsoError;
use crate::Polar;

mod class;
mod class_method;
mod from_polar;
mod method;
mod to_polar;
mod value;

pub use class::{Class, ClassBuilder, Instance};
pub use from_polar::{FromPolar, FromPolarList};
pub use to_polar::{PolarIterator, ToPolar, ToPolarList};
pub use value::PolarValue;

lazy_static::lazy_static! {
    /// Map of classes that have been globally registered
    ///
    /// These will be used as a fallback, and cached on the host when an unknown instance is seen
    static ref DEFAULT_CLASSES: Arc<RwLock<HashMap<std::any::TypeId, super::Class>>> = Default::default();
}

impl crate::PolarClass for Class {}

fn metaclass() -> Class {
    Class::builder::<Class>().name("oso::host::Class").build()
}

/// Maintain mappings and caches for Rust classes & instances
#[derive(Clone)]
pub struct Host {
    /// Reference to the inner `Polar` instance
    polar: Arc<Polar>,

    /// Map from names to `Class`s
    classes: HashMap<String, Class>,

    /// Map of cached instances
    instances: HashMap<u64, class::Instance>,

    /// Map from type IDs, to class names
    /// This helps us go from a generic type `T` to the
    /// class name it is registered as
    class_names: HashMap<std::any::TypeId, String>,
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
        let name = type_class.name.clone();
        host.cache_class(type_class, name)
            .expect("could not register the metaclass");
        host
    }

    pub fn get_class(&self, name: &str) -> crate::Result<&Class> {
        self.classes
            .get(name)
            .ok_or_else(|| OsoError::MissingClassError {
                name: name.to_string(),
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

    pub fn get_class_mut(&mut self, name: &str) -> crate::Result<&mut Class> {
        self.classes
            .get_mut(name)
            .ok_or_else(|| OsoError::MissingClassError {
                name: name.to_string(),
            })
    }

    /// Add the class to the host classes
    ///
    /// Returns an instance of `Type` for this class.
    pub fn cache_class(&mut self, class: Class, name: String) -> crate::Result<String> {
        // Insert into default classes here so that we don't repeat this the first
        // time we see an instance.
        DEFAULT_CLASSES
            .write()
            .unwrap()
            .entry(class.type_id)
            .or_insert_with(|| class.clone());

        if self.classes.contains_key(&name) {
            Err(OsoError::DuplicateClassError { name })
        } else {
            self.class_names.insert(class.type_id, name.clone());
            self.classes.insert(name.clone(), class);
            Ok(name)
        }
    }

    pub fn get_instance(&self, id: u64) -> crate::Result<&class::Instance> {
        tracing::trace!("instances: {:?}", self.instances.keys().collect::<Vec<_>>());
        self.instances
            .get(&id)
            .ok_or(OsoError::MissingInstanceError)
    }

    pub fn cache_instance(&mut self, instance: class::Instance, id: Option<u64>) -> u64 {
        // Lookup the class for this instance
        let type_id = instance.type_id();
        let class = self.get_class_by_type_id(type_id);
        if class.is_err() {
            // if its not found, try and use the default class implementation
            let default_class = DEFAULT_CLASSES.read().unwrap().get(&type_id).cloned();
            if let Some(class) = default_class {
                let name = class.name.clone();
                let _ = self.cache_class(class, name);
            }
        }

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
        name: &str,
        fields: Vec<PolarValue>,
        id: u64,
    ) -> crate::Result<()> {
        let class = self.get_class(name)?.clone();
        debug_assert!(self.instances.get(&id).is_none());
        let fields = fields;
        let instance = class.init(fields)?;
        self.cache_instance(instance, Some(id));
        Ok(())
    }

    pub fn unify(&self, left: u64, right: u64) -> crate::Result<bool> {
        tracing::trace!("unify {:?}, {:?}", left, right);

        let left = self.get_instance(left).unwrap();
        let right = self.get_instance(right).unwrap();
        left.equals(right, &self)
    }

    pub fn isa(&self, value: PolarValue, class_tag: &str) -> crate::Result<bool> {
        let res = match value {
            PolarValue::Instance(instance) => {
                let class = self.get_class(class_tag)?;
                instance.instance_of(class)
            }
            PolarValue::Boolean(_) => class_tag == "Boolean",
            PolarValue::Map(_) => class_tag == "Dictionary",
            PolarValue::List(_) => class_tag == "List",
            PolarValue::Integer(_) => class_tag == "Integer",
            PolarValue::Float(_) => class_tag == "Float",
            PolarValue::String(_) => class_tag == "String",
            _ => false,
        };
        Ok(res)
    }

    pub fn is_subspecializer(&self, _id: u64, _left_tag: &str, _right_tag: &str) -> bool {
        // Rust has no notion of inheritance, so there are no subspecializers.
        false
    }

    pub fn operator(
        &self,
        _op: polar_core::terms::Operator,
        _args: [class::Instance; 2],
    ) -> crate::Result<bool> {
        // Operators are not supported
        // TODO (dhatch): Implement.
        Err(OsoError::UnimplementedOperation {
            operation: String::from("comparison operators"),
        })
    }
}
