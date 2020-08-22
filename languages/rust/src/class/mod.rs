use super::*;

mod method;
use method::*;

#[derive(Clone)]
pub struct Class {
    pub name: String,
    pub constructor: Constructor,
    pub attributes: InstanceMethods,
    pub instance_methods: InstanceMethods,
    pub class_methods: ClassMethods,
    is_check: Arc<dyn Fn(&dyn Any) -> bool>,
}

impl Class {
    pub fn new<T: std::default::Default + 'static>() -> Self {
        Self::with_constructor::<T, _, _>(T::default)
    }

    pub fn with_constructor<T, F, Args>(f: F) -> Self
    where
        T: 'static,
        F: Function<Args, Result = T> + 'static,
        Args: FromPolar + 'static,
    {
        Self {
            name: std::any::type_name::<Self>().to_string(),
            constructor: Constructor::new(f),
            attributes: InstanceMethods::new(),
            instance_methods: InstanceMethods::new(),
            class_methods: ClassMethods::new(),
            is_check: Arc::new(|any| any.downcast_ref::<T>().is_some()),
        }
    }

    pub fn add_attribute_getter<T, F, R>(&mut self, name: &str, f: F)
    where
        F: Method<T, Result = R> + 'static,
        F::Result: ToPolar + 'static,
        T: 'static,
    {
        self.attributes
            .insert(Name(name.to_string()), InstanceMethod::new(f));
    }
    pub fn add_method<T, F, Args, R>(&mut self, name: &str, f: F)
    where
        Args: FromPolar,
        F: Method<T, Args, Result = R> + 'static,
        F::Result: ToPolar + 'static,
        T: 'static,
    {
        self.instance_methods
            .insert(Name(name.to_string()), InstanceMethod::new(f));
    }

    pub fn add_class_method<F, Args, R>(&mut self, name: &str, f: F)
    where
        F: Function<Args, Result = R> + 'static,
        Args: FromPolar + 'static,
        R: ToPolar + 'static,
    {
        self.class_methods
            .insert(Name(name.to_string()), ClassMethod::new(f));
    }

    pub fn register(
        self,
        name: Option<String>,
        polar: &mut crate::polar::Polar,
    ) -> anyhow::Result<()> {
        polar.register_class(self, name)?;
        Ok(())
    }

    pub fn isinstance(&self, instance: &dyn Any) -> bool {
        (self.is_check)(instance)
    }
}

#[derive(Clone)]
pub struct Instance {
    pub instance: Arc<dyn Any>,
    pub attributes: Arc<InstanceMethods>,
    pub methods: Arc<InstanceMethods>,
}
