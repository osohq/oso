//! Wrapper structs for the generic `Function` and `Method` traits
use polar_core::terms::Term;

use std::any::Any;
use std::sync::Arc;

use super::to_polar::ToPolarResults;
use crate::host::from_polar::FromPolarList;
use crate::host::to_polar::{PolarIter, PolarResultIter};

use super::class::Class;
use super::downcast;
use super::method::{Function, Method};
use super::Host;

fn join<A, B>(left: crate::Result<A>, right: crate::Result<B>) -> crate::Result<(A, B)> {
    left.and_then(|l| right.map(|r| (l, r)))
}

type TypeErasedFunction<R> = Arc<dyn Fn(Vec<Term>, &mut Host) -> crate::Result<R> + Send + Sync>;
type TypeErasedMethod<R> =
    Arc<dyn Fn(&dyn Any, Vec<Term>, &mut Host) -> crate::Result<R> + Send + Sync>;

#[derive(Clone)]
pub struct Constructor(TypeErasedFunction<Arc<dyn Any>>);

impl Constructor {
    pub fn new<Args, F>(f: F) -> Self
    where
        Args: FromPolarList,
        F: Function<Args>,
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
pub struct AttributeGetter(
    pub Arc<dyn Fn(&dyn Any, &mut Host) -> crate::Result<Term> + Send + Sync>,
);

impl AttributeGetter {
    pub fn new<T, F, R>(f: F) -> Self
    where
        T: 'static,
        F: Fn(&T) -> R + Send + Sync + 'static,
        R: crate::ToPolar,
    {
        Self(Arc::new(move |receiver, host: &mut Host| {
            let receiver = downcast(receiver).map_err(|e| e.invariant().into());
            receiver.map(&f).map(|v| v.to_polar(host))
        }))
    }
}

#[derive(Clone)]
pub struct InstanceMethod(TypeErasedMethod<PolarResultIter>);

impl InstanceMethod {
    pub fn new<T, F, Args>(f: F) -> Self
    where
        Args: FromPolarList,
        F: Method<T, Args>,
        F::Result: ToPolarResults,
        T: 'static,
    {
        Self(Arc::new(
            move |receiver: &dyn Any, args: Vec<Term>, host: &mut Host| {
                let receiver = downcast(receiver).map_err(|e| e.invariant().into());

                let args = Args::from_polar_list(&args, host);

                join(receiver, args)
                    .map(|(receiver, args)| f.invoke(receiver, args).to_polar_results(host))
            },
        ))
    }

    pub fn new_iterator<T, F, Args, I>(f: F) -> Self
    where
        Args: FromPolarList,
        F: Method<T, Args>,
        F::Result: IntoIterator<Item = I>,
        <<F as Method<T, Args>>::Result as IntoIterator>::IntoIter: Sized + 'static,
        I: ToPolarResults + 'static,
        T: 'static,
    {
        Self(Arc::new(
            move |receiver: &dyn Any, args: Vec<Term>, host: &mut Host| {
                let receiver = downcast(receiver).map_err(|e| e.invariant().into());

                let args = Args::from_polar_list(&args, host);

                join(receiver, args).map(|(receiver, args)| {
                    let polar_values = PolarIter {
                        iter: f.invoke(receiver, args).into_iter(),
                    };
                    polar_values.to_polar_results(host)
                })
            },
        ))
    }

    pub fn invoke(
        &self,
        receiver: &dyn Any,
        args: Vec<Term>,
        host: &mut Host,
    ) -> crate::Result<PolarResultIter> {
        self.0(receiver, args, host)
    }

    pub fn from_class_method(name: String) -> Self {
        Self(Arc::new(
            move |receiver: &dyn Any, args: Vec<Term>, host: &mut Host| {
                downcast::<Class>(receiver)
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
pub struct ClassMethod(TypeErasedFunction<PolarResultIter>);

impl ClassMethod {
    pub fn new<F, Args>(f: F) -> Self
    where
        Args: FromPolarList,
        F: Function<Args> + 'static,
        F::Result: ToPolarResults + 'static,
    {
        Self(Arc::new(move |args: Vec<Term>, host: &mut Host| {
            Args::from_polar_list(&args, host).map(|args| f.invoke(args).to_polar_results(host))
        }))
    }

    pub fn invoke(&self, args: Vec<Term>, host: &mut Host) -> crate::Result<PolarResultIter> {
        self.0(args, host)
    }
}
