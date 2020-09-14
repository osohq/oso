//! Support for dynamic class objects in Rust

use polar_core::terms::{Symbol, Term};

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt;
use std::fmt::Debug;
use std::sync::Arc;

use crate::{FromPolar, ToPolar};

use super::class_method::{ClassMethod, Constructor, InstanceMethod};
use super::method::{Function, Method};
use super::Host;

type ClassMethods = HashMap<Symbol, ClassMethod>;
type InstanceMethods = HashMap<Symbol, InstanceMethod>;

#[derive(Clone)]
pub struct Class<T = ()> {
    /// The class name. Defaults to the `std::any::type_name`
    pub name: String,
    /// A wrapped method that constructs an instance of `T` from Polar terms
    pub constructor: Option<Constructor>,
    /// Methods that return simple attribute lookups on an instance of `T`
    pub attributes: InstanceMethods,
    /// Instance methods on `T` that expect Polar terms, and an instance of `&T`
    pub instance_methods: InstanceMethods,
    /// Class methods on `T`
    pub class_methods: ClassMethods,
    pub type_id: TypeId,
    /// A method to check whether the supplied argument is in instance of `T`
    instance_check: Arc<dyn Fn(&dyn Any) -> bool>,
    /// A method to check whether the supplied `TypeId` matches this class
    /// (This isn't using `type_id` because we might want to register other types here
    /// in order to check inheritance)
    class_check: Arc<dyn Fn(TypeId) -> bool>,

    /// A type marker. This is erased when the class is ready to be constructed with
    /// `erase_type`
    ty: std::marker::PhantomData<T>,
}

impl fmt::Debug for Class {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Class")
            .field("name", &self.name)
            .field("type_id", &self.type_id)
            .finish()
    }
}

impl Default for Class {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Class<T> {
    pub fn new() -> Self
    where
        T: 'static,
    {
        Self {
            name: std::any::type_name::<Self>().to_string(),
            constructor: None,
            attributes: InstanceMethods::new(),
            instance_methods: InstanceMethods::new(),
            class_methods: ClassMethods::new(),
            instance_check: Arc::new(|any| any.is::<T>()),
            class_check: Arc::new(|type_id| TypeId::of::<T>() == type_id),
            ty: std::marker::PhantomData,
            type_id: TypeId::of::<T>(),
        }
    }

    pub fn with_default() -> Self
    where
        T: std::default::Default + 'static,
    {
        Self::with_constructor::<_, _>(T::default)
    }

    pub fn with_constructor<F, Args>(f: F) -> Self
    where
        T: 'static,
        F: Function<Args, Result = T> + 'static,
        Args: FromPolar + 'static,
    {
        let mut class: Class<T> = Class::new();
        class = class.set_constructor(f);
        class
    }

    pub fn set_constructor<F, Args>(mut self, f: F) -> Self
    where
        T: 'static,
        F: Function<Args, Result = T> + 'static,
        Args: FromPolar + 'static,
    {
        self.constructor = Some(Constructor::new(f));
        self
    }

    pub fn add_attribute_getter<F, R>(mut self, name: &str, f: F) -> Self
    where
        F: Method<T, Result = R> + 'static,
        R: ToPolar + 'static,
        T: 'static,
    {
        self.attributes
            .insert(Symbol(name.to_string()), InstanceMethod::new(f));
        self
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn add_method<F, Args, R>(mut self, name: &str, f: F) -> Self
    where
        Args: FromPolar,
        F: Method<T, Args, Result = R> + 'static,
        R: ToPolar + 'static,
        T: 'static,
    {
        self.instance_methods
            .insert(Symbol(name.to_string()), InstanceMethod::new(f));
        self
    }

    pub fn add_result_method<F, Args, R, E>(mut self, name: &str, f: F) -> Self
    where
        Args: FromPolar,
        F: Method<T, Args, Result = Result<R, E>> + 'static,
        R: ToPolar + 'static,
        E: Debug + 'static,
        T: 'static,
    {
        self.instance_methods
            .insert(Symbol(name.to_string()), InstanceMethod::new_result(f));
        self
    }

    pub fn add_class_method<F, Args, R>(mut self, name: &str, f: F) -> Self
    where
        F: Function<Args, Result = R> + 'static,
        Args: FromPolar + 'static,
        R: ToPolar + 'static,
    {
        self.class_methods
            .insert(Symbol(name.to_string()), ClassMethod::new(f));
        self
    }

    /// Erase the generic type parameter
    /// This is done before registering so
    /// that the host can store all of the same type. The generic paramtere
    /// is just used for the builder pattern part of Class
    /// TODO: Skip this shenanigans and make there a builder instead?
    pub fn erase_type(self) -> Class<()> {
        Class {
            name: self.name,
            constructor: self.constructor,
            attributes: self.attributes,
            instance_methods: self.instance_methods,
            class_methods: self.class_methods,
            instance_check: self.instance_check,
            class_check: self.class_check,
            type_id: self.type_id,
            ty: std::marker::PhantomData,
        }
    }

    pub fn build(self) -> Class<()> {
        self.erase_type()
    }

    pub fn is_class<C: 'static>(&self) -> bool {
        tracing::trace!(
            input = %std::any::type_name::<C>(),
            class = %self.name,
            "is_class"
        );
        (self.class_check)(TypeId::of::<C>())
    }

    pub fn is_instance(&self, instance: &Instance) -> bool {
        tracing::trace!(
            instance = %instance.name,
            class = %self.name,
            "is_instance"
        );
        (self.instance_check)(instance.instance.as_ref())
    }

    pub fn init(&self, fields: Vec<Term>, host: &mut Host) -> crate::Result<Instance> {
        if let Some(constructor) = &self.constructor {
            let instance = constructor.invoke(fields, host)?;
            Ok(Instance {
                name: self.name.clone(),
                instance,
                attributes: Arc::new(self.attributes.clone()),
                methods: Arc::new(self.instance_methods.clone()),
            })
        } else {
            Err(crate::OsoError::Custom {
                message: format!("MissingConstructorError: {} has no constructor", self.name),
            })
        }
    }

    pub fn cast_to_instance(&self, instance: impl Any) -> Instance {
        Instance {
            name: self.name.clone(),
            instance: Arc::new(instance),
            attributes: Arc::new(self.attributes.clone()),
            methods: Arc::new(self.instance_methods.clone()),
        }
    }
}

#[derive(Clone)]
pub struct Instance {
    pub name: String,
    pub instance: Arc<dyn Any>,
    pub attributes: Arc<InstanceMethods>,
    pub methods: Arc<InstanceMethods>,
}
