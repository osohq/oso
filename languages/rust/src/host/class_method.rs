use polar_core::terms::{Symbol, Term};

use std::any::Any;
use std::sync::Arc;

use crate::{FromPolar, ToPolar};

use super::class::Class;
use super::method::{Function, Method};
use super::Host;

#[derive(Clone)]
pub struct Constructor(Arc<dyn Fn(Vec<Term>, &mut Host) -> Arc<dyn Any>>);

impl Constructor {
    pub fn new<Args, F>(f: F) -> Self
    where
        Args: FromPolar,
        F: Function<Args> + 'static,
        F::Result: 'static,
    {
        Constructor(Arc::new(move |args: Vec<Term>, host: &mut Host| {
            let args = Args::from_polar_list(&args, host).unwrap();
            Arc::new(f.invoke(args))
        }))
    }

    pub fn invoke(&self, args: Vec<Term>, host: &mut Host) -> Arc<dyn Any> {
        self.0(args, host)
    }
}

#[derive(Clone)]
pub struct InstanceMethod(Arc<dyn Fn(&dyn Any, Vec<Term>, &mut Host) -> Arc<dyn ToPolar>>);

impl InstanceMethod {
    pub fn new<T, F, Args>(f: F) -> Self
    where
        Args: FromPolar,
        F: Method<T, Args> + 'static,
        F::Result: ToPolar + 'static,
        T: 'static,
    {
        Self(Arc::new(
            move |receiver: &dyn Any, args: Vec<Term>, host: &mut Host| {
                let receiver = receiver
                    .downcast_ref()
                    .expect("incorrect type for receiver");
                let args = Args::from_polar_list(&args, host).unwrap();
                Arc::new(f.invoke(receiver, args))
            },
        ))
    }

    pub fn invoke(&self, receiver: &dyn Any, args: Vec<Term>, host: &mut Host) -> Arc<dyn ToPolar> {
        self.0(receiver, args, host)
    }

    pub fn from_class_method(name: Symbol) -> Self {
        Self(Arc::new(
            move |receiver: &dyn Any, args: Vec<Term>, host: &mut Host| {
                let class: &Class = receiver.downcast_ref().unwrap();
                tracing::trace!(class = %class.name, method=%name, "class_method");
                let class_method: &ClassMethod =
                    class.class_methods.get(&name).expect("get class method");
                class_method.invoke(args, host)
            },
        ))
    }
}

#[derive(Clone)]
pub struct ClassMethod(Arc<dyn Fn(Vec<Term>, &mut Host) -> Arc<dyn ToPolar>>);

impl ClassMethod {
    pub fn new<F, Args>(f: F) -> Self
    where
        Args: FromPolar,
        F: Function<Args> + 'static,
        F::Result: ToPolar + 'static,
    {
        Self(Arc::new(move |args: Vec<Term>, host: &mut Host| {
            let args = Args::from_polar_list(&args, host).unwrap();
            Arc::new(f.invoke(args))
        }))
    }

    pub fn invoke(&self, args: Vec<Term>, host: &mut Host) -> Arc<dyn ToPolar> {
        self.0(args, host)
    }
}
