//! Support for dynamic class objects in Rust

use polar_core::terms::Term;

use std::any::TypeId;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use crate::errors::{InvalidCallError, OsoError};

use super::class_method::{AttributeGetter, ClassMethod, Constructor, InstanceMethod};
use super::from_polar::FromPolarList;
use super::method::{Function, Method};
use super::to_polar::ToPolarResults;
use super::Host;

type Attributes = HashMap<&'static str, AttributeGetter>;
type ClassMethods = HashMap<&'static str, ClassMethod>;
type InstanceMethods = HashMap<&'static str, InstanceMethod>;

fn equality_not_supported(
    type_name: String,
) -> Box<dyn Fn(&Instance, &Instance) -> crate::Result<bool> + Send + Sync> {
    let eq = move |_: &Instance, _: &Instance| -> crate::Result<bool> {
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
    equality_check: Arc<dyn Fn(&Instance, &Instance) -> crate::Result<bool> + Send + Sync>,
}

impl Class {
    pub fn builder<T: 'static>() -> ClassBuilder<T> {
        ClassBuilder::new()
    }

    pub fn init(&self, fields: Vec<Term>, host: &mut Host) -> crate::Result<Instance> {
        if let Some(constructor) = &self.constructor {
            constructor.invoke(fields, host)
        } else {
            Err(crate::OsoError::Custom {
                message: format!("MissingConstructorError: {} has no constructor", self.name),
            })
        }
    }

    /// Call class method `attr` on `self` with arguments from `args`.
    ///
    /// Returns: An iterable of results from the method.
    pub fn call(
        &self,
        attr: &str,
        args: Vec<Term>,
        host: &mut Host,
    ) -> crate::Result<super::to_polar::PolarResultIter> {
        let attr =
            self.class_methods
                .get(attr)
                .ok_or_else(|| InvalidCallError::ClassMethodNotFound {
                    method_name: attr.to_owned(),
                    type_name: self.name.clone(),
                })?;

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
        let short_name = fq_name.split("::").last().expect("type has invalid name");
        Self {
            class: Class {
                name: short_name.to_string(),
                constructor: None,
                attributes: HashMap::new(),
                instance_methods: InstanceMethods::new(),
                class_methods: ClassMethods::new(),
                class_check: Arc::new(|type_id| TypeId::of::<T>() == type_id),
                equality_check: Arc::from(equality_not_supported(short_name.to_string())),
                type_id: TypeId::of::<T>(),
            },
            ty: std::marker::PhantomData,
        }
    }

    /// Create a new class builder for a type that implements Default and use that as the constructor.
    pub fn with_default() -> Self
    where
        T: std::default::Default,
        T: Send + Sync,
    {
        Self::with_constructor::<_, _>(T::default)
    }

    /// Create a new class builder with a given constructor.
    pub fn with_constructor<F, Args>(f: F) -> Self
    where
        F: Function<Args, Result = T>,
        T: Send + Sync,
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
        T: Send + Sync,
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

            let a = a.downcast().map_err(|e| e.user())?;
            let b = b.downcast().map_err(|e| e.user())?;

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

/// Container for an instance of a `Class`
///
/// Not guaranteed to be an instance of a registered class,
/// this is looked up through the `Instance::class` method,
/// and the `ToPolar` implementation for `PolarClass` will
/// register the class if not seen before.
///
/// A reference to the underlying type of the Instance can be
/// retrived using `Instance::downcast`.
#[derive(Clone)]
pub struct Instance {
    inner: Arc<dyn std::any::Any + Send + Sync>,

    /// The type name of the Instance, to be used for debugging purposes only.
    /// To get the registered name, use `Instance::name`.
    debug_type_name: &'static str,
}

impl fmt::Debug for Instance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Instance<{}>", self.debug_type_name)
    }
}

impl Instance {
    /// Create a new instance
    pub fn new<T: Send + Sync + 'static>(instance: T) -> Self {
        Self {
            inner: Arc::new(instance),
            debug_type_name: std::any::type_name::<T>(),
        }
    }

    /// Check whether this is an instance of `class`
    pub fn instance_of(&self, class: &Class) -> bool {
        self.inner.as_ref().type_id() == class.type_id
    }

    /// Looks up the `Class` for this instance on the provided `host`
    pub fn class<'a>(&self, host: &'a Host) -> crate::Result<&'a Class> {
        host.get_class_by_type_id(self.inner.as_ref().type_id())
            .map_err(|_| OsoError::MissingClassError {
                name: self.debug_type_name.to_string(),
            })
    }

    /// Get the registered name of this instance on ``host``.
    pub fn name<'a>(&self, host: &'a Host) -> crate::Result<&'a str> {
        Ok(self.class(host)?.name.as_ref())
    }

    /// Lookup an attribute on the instance via the registered `Class`
    pub fn get_attr(&self, name: &str, host: &mut Host) -> crate::Result<Term> {
        tracing::trace!({ method = %name }, "get_attr");
        let attr = self
            .class(host)
            .and_then(|c| {
                c.attributes.get(name).ok_or_else(|| {
                    InvalidCallError::AttributeNotFound {
                        attribute_name: name.to_owned(),
                        type_name: self.debug_type_name.to_owned(),
                    }
                    .into()
                })
            })?
            .clone();
        attr.invoke(self, host)
    }

    /// Call the named method on the instance via the registered `Class`
    ///
    /// Returns: PolarResultIter, or an Error if the method cannot be called.
    ///
    /// N.B: If the method itself returns an error, this will be captured in
    /// the PolarResultIterator (the first result will be an Error).
    pub fn call(
        &self,
        name: &str,
        args: Vec<Term>,
        host: &mut Host,
    ) -> crate::Result<super::to_polar::PolarResultIter> {
        tracing::trace!({method = %name, ?args}, "call");
        let method = self.class(host).and_then(|c| {
            c.get_method(name).ok_or_else(|| {
                InvalidCallError::MethodNotFound {
                    method_name: name.to_owned(),
                    type_name: self.debug_type_name.to_owned(),
                }
                .into()
            })
        })?;
        method.invoke(self, args, host)
    }

    /// Return `true` if the `instance` of self equals the instance of `other`.
    pub fn equals(&self, other: &Self, host: &Host) -> crate::Result<bool> {
        tracing::trace!("equals");
        self.class(host)
            .and_then(|c| (c.equality_check)(&self, &other))
    }

    /// Attempt to downcast the inner type of the instance to a reference to the type `T`
    /// This should be the _only_ place using downcast to avoid mistakes.
    pub fn downcast<T: 'static>(&self) -> Result<&T, crate::errors::TypeError> {
        self.inner
            .as_ref()
            .downcast_ref()
            .ok_or_else(|| crate::errors::TypeError {
                expected: String::from(std::any::type_name::<T>()),
            })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_instance_of() {
        struct Foo {}
        struct Bar {}

        let foo_class = Class::builder::<Foo>().build();
        let bar_class = Class::builder::<Bar>().build();
        let foo_instance = Instance::new(Foo {});

        assert!(foo_instance.instance_of(&foo_class));
        assert!(!foo_instance.instance_of(&bar_class));
    }
}
