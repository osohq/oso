//! Support for dynamic class objects in Rust

use polar_core::terms::Term;

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use crate::errors::OsoError;

use super::class_method::{AttributeGetter, ClassMethod, Constructor, InstanceMethod};
use super::downcast;
use super::from_polar::FromPolarList;
use super::method::{Function, Method};
use super::to_polar::ToPolarResults;
use super::Host;

type Attributes = HashMap<&'static str, AttributeGetter>;
type ClassMethods = HashMap<&'static str, ClassMethod>;
type InstanceMethods = HashMap<&'static str, InstanceMethod>;

fn equality_not_supported(
    type_name: String,
) -> Box<dyn Fn(&dyn Any, &dyn Any) -> crate::Result<bool> + Send + Sync> {
    let eq = move |_: &dyn Any, _: &dyn Any| -> crate::Result<bool> {
        Err(OsoError::UnsupportedOperation {
            operation: String::from("equals"),
            type_name: type_name.clone(),
        })
    };

    Box::new(eq)
}

#[derive(Clone)]
pub struct Class {
    /// The class name. Defaults to the `std::any::type_name`
    pub name: String,
    pub type_id: TypeId,
    /// A wrapped method that constructs an instance of `T` from Polar terms
    constructor: Option<Constructor>,
    /// Methods that return simple attribute lookups on an instance of `T`
    attributes: Attributes,
    /// Instance methods on `T` that expect Polar terms, and an instance of `&T`
    instance_methods: InstanceMethods,
    /// Class methods on `T`
    class_methods: ClassMethods,

    /// A method to check whether the supplied `TypeId` matches this class
    /// (This isn't using `type_id` because we might want to register other types here
    /// in order to check inheritance)
    class_check: Arc<dyn Fn(TypeId) -> bool + Send + Sync>,

    /// A function that accepts arguments of this class and compares them for equality.
    /// Limitation: Only works on comparisons of the same type.
    equality_check: Arc<dyn Fn(&dyn Any, &dyn Any) -> crate::Result<bool> + Send + Sync>,
}

impl Class {
    pub fn builder<T: 'static>() -> ClassBuilder<T> {
        ClassBuilder::new()
    }

    pub fn init(&self, fields: Vec<Term>, host: &mut Host) -> crate::Result<Instance> {
        if let Some(constructor) = &self.constructor {
            let instance = constructor.invoke(fields, host)?;
            Ok(Instance {
                ty: instance.as_ref().type_id(),
                inner: instance,
            })
        } else {
            Err(crate::OsoError::Custom {
                message: format!("MissingConstructorError: {} has no constructor", self.name),
            })
        }
    }

    pub fn call(
        &self,
        attr: &str,
        args: Vec<Term>,
        host: &mut Host,
    ) -> crate::Result<super::to_polar::PolarResultIter> {
        let attr = self
            .class_methods
            .get(attr)
            .expect("class method not found");
        attr.clone().invoke(args, host)
    }

    fn get_method(&self, name: &str) -> Option<InstanceMethod> {
        tracing::trace!({class=%self.name, name}, "get_method");
        if self.type_id == TypeId::of::<Class>() {
            // all methods on `Class` redirect by looking up the class method
            Some(InstanceMethod::from_class_method(name.to_string()))
        } else {
            self.instance_methods.get(name).cloned()
        }
    }
}

#[derive(Clone)]
pub struct ClassBuilder<T> {
    class: Class,
    /// A type marker. Used to ensure methods have the correct type.
    ty: std::marker::PhantomData<T>,
}

impl<T> ClassBuilder<T>
where
    T: 'static,
{
    /// Create a new class builder.
    fn new() -> Self {
        let fq_name = std::any::type_name::<T>().to_string();
        let short_name = fq_name.split("::").last().expect("type has no name");
        Self {
            class: Class {
                name: short_name.to_string(),
                constructor: None,
                attributes: HashMap::new(),
                instance_methods: InstanceMethods::new(),
                class_methods: ClassMethods::new(),
                class_check: Arc::new(|type_id| TypeId::of::<T>() == type_id),
                equality_check: Arc::from(equality_not_supported(fq_name)),
                type_id: TypeId::of::<T>(),
            },
            ty: std::marker::PhantomData,
        }
    }

    /// Create a new class builder for a type that implements Default and use that as the constructor.
    pub fn with_default() -> Self
    where
        T: std::default::Default,
    {
        Self::with_constructor::<_, _>(T::default)
    }

    /// Create a new class builder with a given constructor.
    pub fn with_constructor<F, Args>(f: F) -> Self
    where
        F: Function<Args, Result = T>,
        Args: FromPolarList,
    {
        let mut class: ClassBuilder<T> = ClassBuilder::new();
        class = class.set_constructor(f);
        class
    }

    /// Set the constructor function to use for polar `new` statements.
    pub fn set_constructor<F, Args>(mut self, f: F) -> Self
    where
        F: Function<Args, Result = T>,
        Args: FromPolarList,
    {
        self.class.constructor = Some(Constructor::new(f));
        self
    }

    /// Set an equality function to be used for polar `==` statements.
    pub fn set_equality_check<F>(mut self, f: F) -> Self
    where
        F: Fn(&T, &T) -> bool + Send + Sync + 'static,
    {
        self.class.equality_check = Arc::new(move |a, b| {
            tracing::trace!("equality check");

            let a = downcast(a).map_err(|e| e.user())?;
            let b = downcast(b).map_err(|e| e.user())?;

            Ok((f)(a, b))
        });

        self
    }

    /// Use PartialEq::eq as the equality check for polar `==` statements.
    pub fn with_equality_check(self) -> Self
    where
        T: PartialEq<T>,
    {
        self.set_equality_check(|a, b| PartialEq::eq(a, b))
    }

    /// Add an attribute getter for statments like `foo.bar`
    /// `class.add_attribute_getter("bar", |instance| instance.bar)
    pub fn add_attribute_getter<F, R>(mut self, name: &'static str, f: F) -> Self
    where
        F: Fn(&T) -> R + Send + Sync + 'static,
        R: crate::ToPolar,
        T: 'static,
    {
        self.class.attributes.insert(name, AttributeGetter::new(f));
        self
    }

    /// Set the name of the polar class.
    pub fn name(mut self, name: &str) -> Self {
        self.class.name = name.to_string();
        self
    }

    /// Add a method for polar method calls like `foo.plus(1)
    /// `class.add_attribute_getter("bar", |instance, n| instance.foo + n)
    pub fn add_method<F, Args, R>(mut self, name: &'static str, f: F) -> Self
    where
        Args: FromPolarList,
        F: Method<T, Args, Result = R>,
        R: ToPolarResults + 'static,
    {
        self.class
            .instance_methods
            .insert(name, InstanceMethod::new(f));
        self
    }

    /// A method that returns multiple values. Every element in the iterator returned by the method will
    /// be a separate polar return value.
    pub fn add_iterator_method<F, Args, I>(mut self, name: &'static str, f: F) -> Self
    where
        Args: FromPolarList,
        F: Method<T, Args>,
        F::Result: IntoIterator<Item = I>,
        <<F as Method<T, Args>>::Result as IntoIterator>::IntoIter: Sized + 'static,
        I: ToPolarResults + 'static,
        T: 'static,
    {
        self.class
            .instance_methods
            .insert(name, InstanceMethod::new_iterator(f));
        self
    }

    /// A method that's called on the type instead of an instance.
    /// eg `Foo.pi`
    pub fn add_class_method<F, Args, R>(mut self, name: &'static str, f: F) -> Self
    where
        F: Function<Args, Result = R>,
        Args: FromPolarList,
        R: ToPolarResults + 'static,
    {
        self.class.class_methods.insert(name, ClassMethod::new(f));
        self
    }

    /// Finish building a build the class
    pub fn build(self) -> Class {
        self.class
    }
}

#[derive(Clone)]
pub struct Instance {
    pub inner: Arc<dyn Any>,
    ty: TypeId,
}

impl fmt::Debug for Instance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Instance<{:?}>", self.ty)
    }
}

impl Instance {
    pub fn new<T: 'static>(instance: T) -> Self {
        Self {
            inner: Arc::new(instance),
            ty: TypeId::of::<T>(),
        }
    }

    pub fn instance_of(&self, class: &Class) -> bool {
        self.inner.as_ref().type_id() == class.type_id
    }

    pub fn class<'a>(&self, host: &'a Host) -> Option<&'a Class> {
        host.get_class_by_type_id(self.ty)
    }

    pub fn get_attr(&self, attr: &str, host: &mut Host) -> crate::Result<Term> {
        let attr = self
            .class(host)
            .and_then(|c| c.attributes.get(attr))
            .ok_or_else(|| OsoError::Custom {
                message: format!("attribute {} not found", attr),
            })?;
        (attr.0.clone())(self.inner.as_ref(), host)
    }

    pub fn call(
        &self,
        attr: &str,
        args: Vec<Term>,
        host: &mut Host,
    ) -> crate::Result<super::to_polar::PolarResultIter> {
        let attr = self
            .class(host)
            .and_then(|c| c.get_method(attr))
            .ok_or_else(|| OsoError::Custom {
                message: format!("method {} not found", attr),
            })?;
        attr.invoke(self.inner.as_ref(), args, host)
    }
}

impl Instance {
    /// Return `true` if the `instance` of self equals the instance of `other`.
    pub fn equals(&self, other: &Self, host: &Host) -> crate::Result<bool> {
        tracing::trace!("equals");
        // TODO: LOL this &* below is tricky! Have a function to do this, and make instance not
        // pub.
        if let Some(c) = self.class(host) {
            (c.equality_check)(&*self.inner, &*other.inner)
        } else {
            tracing::warn!("class not found for equality check");
            Ok(false)
        }
    }
}

// @TODO: This is very unsafe.
// Temporary workaround. We need to differentiate between instances which
// _do_ need to be `Send` (e.g. registered as constants on the base `Oso` objects)
// and instances which don't need to be Send (e.g. created/accessed on a single thread for
// just one query).
unsafe impl Send for Instance {}
