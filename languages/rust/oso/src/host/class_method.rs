//! Wrapper structs for the generic `Function` and `Method` traits
use polar_core::terms::{Symbol, Term};

use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;

use crate::{FromPolar, ToPolar};

use super::class::Class;
use super::method::{Function, Method};
use super::Host;

fn join<A, B>(left: crate::Result<A>, right: crate::Result<B>) -> crate::Result<(A, B)> {
    left.and_then(|l| right.map(|r| (l, r)))
}

type TypeErasedFunction<R> = Arc<dyn Fn(Vec<Term>, &mut Host) -> crate::Result<Arc<R>>>;
type TypeErasedMethod<R> = Arc<dyn Fn(&dyn Any, Vec<Term>, &mut Host) -> crate::Result<Arc<R>>>;

#[derive(Clone)]
pub struct Constructor(TypeErasedFunction<dyn Any>);

impl Constructor {
    pub fn new<Args, F>(f: F) -> Self
    where
        Args: FromPolar,
        F: Function<Args> + 'static,
        F::Result: 'static,
    {
        Constructor(Arc::new(move |args: Vec<Term>, host: &mut Host| {
            Args::from_polar_list(&args, host).map(|args| Arc::new(f.invoke(args)) as Arc<dyn Any>)
        }))
    }

    pub fn invoke(&self, args: Vec<Term>, host: &mut Host) -> crate::Result<Arc<dyn Any>> {
        self.0(args, host)
    }
}

#[derive(Clone)]
pub struct InstanceMethod(TypeErasedMethod<dyn ToPolar>);

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
                    .ok_or_else(|| crate::OsoError::InvalidReceiver);

                let args = Args::from_polar_list(&args, host);

                join(receiver, args)
                    .map(|(receiver, args)| Arc::new(f.invoke(receiver, args)) as Arc<dyn ToPolar>)
            },
        ))
    }

    pub fn new_result<T, F, Args, R, E>(f: F) -> Self
    where
        Args: FromPolar,
        F: Method<T, Args, Result = Result<R, E>> + 'static,
        R: ToPolar + 'static,
        E: Debug + 'static,
        T: 'static,
    {
        Self(Arc::new(
            move |receiver: &dyn Any, args: Vec<Term>, host: &mut Host| {
                let receiver = receiver
                    .downcast_ref()
                    .ok_or_else(|| crate::OsoError::InvalidReceiver);

                let args = Args::from_polar_list(&args, host);

                join(receiver, args).and_then(|(receiver, args)| match f.invoke(receiver, args) {
                    Ok(result) => Ok(Arc::new(result) as Arc<dyn ToPolar>),
                    Err(e) => Err(crate::OsoError::Custom {
                        message: format!("Error calling function: {:?}", e),
                    }),
                })
            },
        ))
    }

    pub fn invoke(
        &self,
        receiver: &dyn Any,
        args: Vec<Term>,
        host: &mut Host,
    ) -> crate::Result<Arc<dyn ToPolar>> {
        self.0(receiver, args, host)
    }

    pub fn from_class_method(name: Symbol) -> Self {
        Self(Arc::new(
            move |receiver: &dyn Any, args: Vec<Term>, host: &mut Host| {
                receiver
                    .downcast_ref::<Class>()
                    .ok_or_else(|| crate::OsoError::InvalidReceiver)
                    .and_then(|class| {
                        tracing::trace!(class = %class.name, method=%name, "class_method");
                        class
                            .class_methods
                            .get(&name)
                            .ok_or_else(|| crate::OsoError::MethodNotFound)
                    })
                    .and_then(|class_method: &ClassMethod| class_method.invoke(args, host))
            },
        ))
    }
}

#[derive(Clone)]
pub struct ClassMethod(TypeErasedFunction<dyn ToPolar>);

impl ClassMethod {
    pub fn new<F, Args>(f: F) -> Self
    where
        Args: FromPolar,
        F: Function<Args> + 'static,
        F::Result: ToPolar + 'static,
    {
        Self(Arc::new(move |args: Vec<Term>, host: &mut Host| {
            Args::from_polar_list(&args, host)
                .map(|args| Arc::new(f.invoke(args)) as Arc<dyn ToPolar>)
        }))
    }

    pub fn invoke(&self, args: Vec<Term>, host: &mut Host) -> crate::Result<Arc<dyn ToPolar>> {
        self.0(args, host)
    }
}
