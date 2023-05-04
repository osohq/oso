//! Support for dynamic class objects in Rust

use std::any::TypeId;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use crate::errors::{InvalidCallError, OsoError};

use super::class_method::{
    AttributeGetter, ClassMethod, Constructor, InstanceMethod, RegisterHook,
};
use super::from_polar::FromPolarList;
use super::method::{Function, Method};
use super::to_polar::ToPolarResult;
use super::Host;
use super::PolarValue;

type Attributes = HashMap<&'static str, AttributeGetter>;
type RegisterHooks = Vec<RegisterHook>;
type ClassMethods = HashMap<&'static str, ClassMethod>;
type InstanceMethods = HashMap<&'static str, InstanceMethod>;

type EqualityMethod = Arc<dyn Fn(&Host, &Instance, &Instance) -> crate::Result<bool> + Send + Sync>;
type IteratorMethod =
    Arc<dyn Fn(&Host, &Instance) -> crate::Result<crate::host::PolarIterator> + Send + Sync>;

fn equality_not_supported() -> EqualityMethod {
    let eq = move |host: &Host, lhs: &Instance, _: &Instance| -> crate::Result<bool> {
        Err(OsoError::UnsupportedOperation {
            operation: String::from("equals"),
            type_name: lhs.name(host).to_owned(),
        })
    };

    Arc::new(eq)
}

fn iterator_not_supported() -> IteratorMethod {
    let into_iter = move |host: &Host, instance: &Instance| {
        Err(OsoError::UnsupportedOperation {
            operation: String::from("in"),
            type_name: instance.name(host).to_owned(),
        })
    };

    Arc::new(into_iter)
}

/// Class that can be registered with [`Oso`](crate::Oso).
///
/// A class represents an entity, such as an *actor* or a *resource*. It is typically backed by
/// a Rust struct and can carry some internal state, as well as have methods that can be made
/// accessible from within the policy file.
///
/// While the instance of the class itself is stored as a [`PolarValue::Instance`], the [`Class`]
/// struct contains metadata, such as the constructor, attributes, instance methods, comparison
/// functions as well as the name.
#[derive(Clone)]
pub struct Class {
    /// The class name. Defaults to [`std::any::type_name`].
    pub name: String,
    /// Type ID of the class.
    pub type_id: TypeId,
    /// A wrapped method that constructs an instance of `T` from `PolarValue`s
    constructor: Option<Constructor>,
    /// Methods that return simple attribute lookups on an instance of `T`
    attributes: Attributes,
    /// Instance methods on `T` that expect a list of `PolarValue`s, and an instance of `&T`
    instance_methods: InstanceMethods,
    /// Class methods on `T`
    class_methods: ClassMethods,

    /// A function that accepts arguments of this class and compares them for equality.
    /// Limitation: Only works on comparisons of the same type.
    equality_check: EqualityMethod,

    into_iter: IteratorMethod,

    /// Hooks to be called on the class once it's been registered with host.
    pub register_hooks: RegisterHooks,
}

impl Class {
    /// Builder instance to build class.
    ///
    /// Use this when you want to hook your own class into [`Oso`](crate::Oso).
    /// See [`ClassBuilder`] for usage examples.
    pub fn builder<T: 'static>() -> ClassBuilder<T> {
        ClassBuilder::new()
    }

    /// Initialize new class instance.
    pub fn init(&self, fields: Vec<PolarValue>) -> crate::Result<Instance> {
        if let Some(constructor) = &self.constructor {
            constructor.invoke(fields)
        } else {
            Err(crate::OsoError::Custom {
                message: format!("MissingConstructorError: {} has no constructor", self.name),
            })
        }
    }

    /// Call class method `attr` on `self` with arguments from `args`.
    ///
    /// Returns the result as a `PolarValue`.
    pub fn call(&self, attr: &str, args: Vec<PolarValue>) -> crate::Result<PolarValue> {
        let attr =
            self.class_methods
                .get(attr)
                .ok_or_else(|| InvalidCallError::ClassMethodNotFound {
                    method_name: attr.to_owned(),
                    type_name: self.name.clone(),
                })?;

        attr.clone().invoke(args)
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

    fn equals(&self, host: &Host, lhs: &Instance, rhs: &Instance) -> crate::Result<bool> {
        // equality checking is currently only supported for exactly matching types
        // TODO: support multiple dispatch for equality
        if lhs.type_id() != rhs.type_id() {
            Ok(false)
        } else {
            (self.equality_check)(host, lhs, rhs)
        }
    }
}

/// Builder for new Oso [`Class`].
///
/// This helps you create a `Class` instance which holds metadata for your custom type. Using the
/// builder, you can add attribute getters, class methods, instance methods, constants, iterator
/// methods, override the class name, set the constructor or equality check.
///
/// You can create a new instance of [`ClassBuilder`] using
/// [`PolarClass::get_polar_class_builder()`](crate::PolarClass::get_polar_class_builder), using
/// [`Class::builder()`] or using one of the [`ClassBuilder::with_default()`] or
/// [`ClassBuilder::with_constructor()`] methods.
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
                equality_check: equality_not_supported(),
                into_iter: iterator_not_supported(),
                type_id: TypeId::of::<T>(),
                register_hooks: RegisterHooks::new(),
            },
            ty: std::marker::PhantomData,
        }
    }

    /// Create a new class builder for a type that implements [`Default`] and use that as the
    /// constructor.
    ///
    /// This is equivalent to setting the constructor to [`Default::default()`].
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use oso::ClassBuilder;
    ///
    /// #[derive(Default)]
    /// struct MyClass;
    ///
    /// let class = ClassBuilder::<MyClass>::with_default().build();
    /// ```
    pub fn with_default() -> Self
    where
        T: std::default::Default,
        T: Send + Sync,
    {
        Self::with_constructor::<_, _>(T::default)
    }

    /// Create a new class builder with a given constructor.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use oso::ClassBuilder;
    ///
    /// struct MyClass(u16);
    ///
    /// let class = ClassBuilder::<MyClass>::with_constructor(|| MyClass(42)).build();
    ///
    /// ```
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
    ///
    /// # Examples
    ///
    /// ```
    /// use oso::ClassBuilder;
    /// ```
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
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use oso::ClassBuilder;
    ///
    /// #[derive(Default)]
    /// struct MyClass;
    ///
    /// let class = ClassBuilder::<MyClass>::with_default()
    ///     .set_equality_check(|left, right| true)
    ///     .build();
    /// ```
    pub fn set_equality_check<F>(mut self, f: F) -> Self
    where
        F: Fn(&T, &T) -> bool + Send + Sync + 'static,
    {
        self.class.equality_check = Arc::new(move |host, a, b| {
            tracing::trace!("equality check");

            let a = a.downcast(Some(host)).map_err(|e| e.user())?;
            let b = b.downcast(Some(host)).map_err(|e| e.user())?;

            Ok((f)(a, b))
        });

        self
    }

    /// Set a method to convert instances into iterators
    pub fn set_into_iter<F, I, V>(mut self, f: F) -> Self
    where
        F: Fn(&T) -> I + Send + Sync + 'static,
        I: Iterator<Item = V> + Clone + Send + Sync + 'static,
        V: ToPolarResult,
    {
        self.class.into_iter = Arc::new(move |host, instance| {
            tracing::trace!("iter check");

            let instance = instance.downcast(Some(host)).map_err(|e| e.user())?;

            Ok(crate::host::PolarIterator::new(f(instance)))
        });

        self
    }

    /// Use the existing `IntoIterator` implementation to convert instances into iterators
    pub fn with_iter<V>(self) -> Self
    where
        T: IntoIterator<Item = V> + Clone,
        <T as IntoIterator>::IntoIter: Clone + Send + Sync + 'static,
        V: ToPolarResult,
    {
        self.set_into_iter(|t| t.clone().into_iter())
    }

    /// Use [`PartialEq`] as the equality check for Polar `==` statements.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use oso::ClassBuilder;
    ///
    /// #[derive(Default, PartialEq)]
    /// struct MyClass(u64);
    ///
    /// let class = ClassBuilder::<MyClass>::with_default()
    ///     .with_equality_check()
    ///     .build();
    /// ```
    pub fn with_equality_check(self) -> Self
    where
        T: PartialEq<T>,
    {
        self.set_equality_check(|a, b| PartialEq::eq(a, b))
    }

    /// Add an attribute getter.
    ///
    /// An attribute getter allows you to write statements like `foo.bar`, where `foo` is a class
    /// instance and `bar` is an attribute.
    ///
    /// Typically, if you use the [`PolarClass`] derive macro, you can use `#[polar(attribute)]` to
    /// generate this automatically.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use oso::ClassBuilder;
    ///
    /// #[derive(Default)]
    /// struct MyClass {
    ///     name: String,
    ///     age: u32,
    /// };
    ///
    /// let class = ClassBuilder::<MyClass>::with_default()
    ///     .add_attribute_getter("name", |instance| instance.name.clone())
    ///     .add_attribute_getter("age", |instance| instance.age)
    ///     .build();
    /// ```
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

    /// Add a RegisterHook on the class that will register the given constant once the class is registered.
    pub fn add_constant<V: crate::ToPolar + Clone + Send + Sync + 'static>(
        mut self,
        value: V,
        name: &'static str,
    ) -> Self {
        let register_hook = move |oso: &mut crate::Oso| oso.register_constant(value.clone(), name);
        self.class
            .register_hooks
            .push(RegisterHook::new(register_hook));
        self
    }

    /// Add a method for polar method calls like `foo.plus(1)
    /// `class.add_attribute_getter("bar", |instance, n| instance.foo + n)
    pub fn add_method<F, Args, R>(mut self, name: &'static str, f: F) -> Self
    where
        Args: FromPolarList,
        F: Method<T, Args, Result = R>,
        R: ToPolarResult + 'static,
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
        I: ToPolarResult + 'static,
        <<F as Method<T, Args>>::Result as IntoIterator>::IntoIter:
            Iterator<Item = I> + Clone + Send + Sync + 'static,
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
        R: ToPolarResult + 'static,
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
/// retrieved using `Instance::downcast`.
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
        self.type_id() == class.type_id
    }

    pub fn type_id(&self) -> std::any::TypeId {
        self.inner.as_ref().type_id()
    }

    /// Looks up the `Class` for this instance on the provided `host`
    pub fn class<'a>(&self, host: &'a Host) -> crate::Result<&'a Class> {
        host.get_class_by_type_id(self.inner.as_ref().type_id())
            .map_err(|_| OsoError::MissingClassError {
                name: self.debug_type_name.to_string(),
            })
    }

    /// Get the canonical name of this instance.
    ///
    /// The canonical name is the registered name on host *if* if it registered.
    /// Otherwise, the debug name is returned.
    pub fn name<'a>(&self, host: &'a Host) -> &'a str {
        self.class(host)
            .map(|class| class.name.as_ref())
            .unwrap_or_else(|_| self.debug_type_name)
    }

    /// Lookup an attribute on the instance via the registered `Class`
    pub fn get_attr(&self, name: &str, host: &mut Host) -> crate::Result<PolarValue> {
        tracing::trace!({ method = %name }, "get_attr");
        let attr = self
            .class(host)
            .and_then(|c| {
                c.attributes.get(name).ok_or_else(|| {
                    InvalidCallError::AttributeNotFound {
                        attribute_name: name.to_owned(),
                        type_name: self.name(host).to_owned(),
                    }
                    .into()
                })
            })?
            .clone();
        attr.invoke(self, host)
    }

    /// Call the named method on the instance via the registered `Class`
    ///
    /// Returns: A PolarValue, or an Error if the method cannot be called.
    pub fn call(
        &self,
        name: &str,
        args: Vec<PolarValue>,
        host: &mut Host,
    ) -> crate::Result<PolarValue> {
        tracing::trace!({method = %name, ?args}, "call");
        let method = self.class(host).and_then(|c| {
            c.get_method(name).ok_or_else(|| {
                InvalidCallError::MethodNotFound {
                    method_name: name.to_owned(),
                    type_name: self.name(host).to_owned(),
                }
                .into()
            })
        })?;
        method.invoke(self, args, host)
    }

    pub fn as_iter(&self, host: &Host) -> crate::Result<crate::host::PolarIterator> {
        self.class(host).and_then(|c| (c.into_iter)(host, self))
    }

    /// Return `true` if the `instance` of self equals the instance of `other`.
    pub fn equals(&self, other: &Self, host: &Host) -> crate::Result<bool> {
        tracing::trace!("equals");
        self.class(host)
            .and_then(|class| class.equals(host, self, other))
    }

    /// Attempt to downcast the inner type of the instance to a reference to the type `T`
    /// This should be the _only_ place using downcast to avoid mistakes.
    ///
    /// # Arguments
    ///
    /// * `host`: Pass host if possible to improve error handling.
    pub fn downcast<T: 'static>(
        &self,
        host: Option<&Host>,
    ) -> Result<&T, crate::errors::TypeError> {
        let name = host
            .map(|h| self.name(h).to_owned())
            .unwrap_or_else(|| self.debug_type_name.to_owned());

        let expected_name = host
            .and_then(|h| {
                h.get_class_by_type_id(std::any::TypeId::of::<T>())
                    .map(|class| class.name.clone())
                    .ok()
            })
            .unwrap_or_else(|| std::any::type_name::<T>().to_owned());

        self.inner
            .as_ref()
            .downcast_ref()
            .ok_or_else(|| crate::errors::TypeError::expected(expected_name).got(name))
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
