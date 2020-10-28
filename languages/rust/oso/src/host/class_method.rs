//! Wrapper structs for the generic `Function` and `Method` traits
use polar_core::terms::Term;

use std::sync::Arc;

use crate::host::from_polar::FromPolarList;
use crate::host::to_polar::{PolarIterator, ToPolar, ToPolarResult};

use super::class::{Class, Instance};
use super::method::{Function, Method};
use super::Host;

fn join<A, B>(left: crate::Result<A>, right: crate::Result<B>) -> crate::Result<(A, B)> {
    left.and_then(|l| right.map(|r| (l, r)))
}

type TypeErasedFunction<R> = Arc<dyn Fn(Vec<Term>, &mut Host) -> crate::Result<R> + Send + Sync>;
type TypeErasedMethod<R> =
    Arc<dyn Fn(&Instance, Vec<Term>, &mut Host) -> crate::Result<R> + Send + Sync>;

#[derive(Clone)]
pub struct Constructor(TypeErasedFunction<Instance>);

impl Constructor {
    pub fn new<Args, F>(f: F) -> Self
    where
        Args: FromPolarList,
        F: Function<Args>,
        F::Result: Send + Sync,
    {
        Constructor(Arc::new(move |args: Vec<Term>, host: &mut Host| {
            Args::from_polar_list(&args, host).map(|args| Instance::new(f.invoke(args)))
        }))
    }

    pub fn invoke(&self, args: Vec<Term>, host: &mut Host) -> crate::Result<Instance> {
        self.0(args, host)
    }
}

#[derive(Clone)]
pub struct AttributeGetter(Arc<dyn Fn(&Instance, &mut Host) -> crate::Result<Term> + Send + Sync>);

impl AttributeGetter {
    pub fn new<T, F, R>(f: F) -> Self
    where
        T: 'static,
        F: Fn(&T) -> R + Send + Sync + 'static,
        R: ToPolarResult,
    {
        Self(Arc::new(move |receiver, host: &mut Host| {
            let receiver = receiver
                .downcast(Some(&host))
                .map_err(|e| e.invariant().into());
            receiver.map(&f).and_then(|v| v.to_polar_result(host))
        }))
    }

    pub fn invoke(&self, receiver: &Instance, host: &mut Host) -> crate::Result<Term> {
        self.0(receiver, host)
    }
}

#[derive(Clone)]
pub struct InstanceMethod(TypeErasedMethod<Term>);

impl InstanceMethod {
    pub fn new<T, F, Args>(f: F) -> Self
    where
        Args: FromPolarList,
        F: Method<T, Args>,
        F::Result: ToPolarResult,
        T: 'static,
    {
        Self(Arc::new(
            move |receiver: &Instance, args: Vec<Term>, host: &mut Host| {
                let receiver = receiver
                    .downcast(Some(&host))
                    .map_err(|e| e.invariant().into());

                let args = Args::from_polar_list(&args, host);

                join(receiver, args)
                    .and_then(|(receiver, args)| f.invoke(receiver, args).to_polar_result(host))
            },
        ))
    }

    pub fn new_iterator<T, F, Args, I>(f: F) -> Self
    where
        Args: FromPolarList,
        F: Method<T, Args>,
        F::Result: IntoIterator<Item = I>,
        I: ToPolarResult + 'static,
        <<F as Method<T, Args>>::Result as IntoIterator>::IntoIter:
            Iterator<Item = I> + Clone + Send + Sync + 'static,
        T: 'static,
    {
        Self(Arc::new(
            move |receiver: &Instance, args: Vec<Term>, host: &mut Host| {
                let receiver = receiver
                    .downcast(Some(&host))
                    .map_err(|e| e.invariant().into());

                let args = Args::from_polar_list(&args, host);

                join(receiver, args)
                    .map(|(receiver, args)| {
                        PolarIterator::new(f.invoke(receiver, args).into_iter())
                    })
                    .map(|results| results.to_polar(host))
            },
        ))
    }

    pub fn invoke(
        &self,
        receiver: &Instance,
        args: Vec<Term>,
        host: &mut Host,
    ) -> crate::Result<Term> {
        self.0(receiver, args, host)
    }

    pub fn from_class_method(name: String) -> Self {
        Self(Arc::new(
            move |receiver: &Instance, args: Vec<Term>, host: &mut Host| {
                receiver
                    .downcast::<Class>(Some(&host))
                    .map_err(|e| e.invariant().into())
                    .and_then(|class| {
                        tracing::trace!(class = %class.name, method=%name, "class_method");
                        class.call(&name, args, host)
                    })
            },
        ))
    }
}

#[derive(Clone)]
pub struct ClassMethod(TypeErasedFunction<Term>);

impl ClassMethod {
    pub fn new<F, Args>(f: F) -> Self
    where
        Args: FromPolarList,
        F: Function<Args> + 'static,
        F::Result: ToPolarResult + 'static,
    {
        Self(Arc::new(move |args: Vec<Term>, host: &mut Host| {
            Args::from_polar_list(&args, host).and_then(|args| f.invoke(args).to_polar_result(host))
        }))
    }

    pub fn invoke(&self, args: Vec<Term>, host: &mut Host) -> crate::Result<Term> {
        self.0(args, host)
    }
}
