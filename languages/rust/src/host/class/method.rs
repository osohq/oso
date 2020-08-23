use super::*;

pub type ClassMethods = HashMap<Name, ClassMethod>;
pub type InstanceMethods = HashMap<Name, InstanceMethod>;

/// An alternate version of the `Fn` trait
/// which encodes the types of the arguments
/// in a single type - a tuple.
pub trait Function<Args = ()> {
    type Result;

    fn invoke(&self, args: Args) -> Self::Result;
}

impl<F, R> Function<()> for F
where
    F: Fn() -> R,
{
    type Result = R;

    fn invoke(&self, _: ()) -> Self::Result {
        (self)()
    }
}

impl<A, F, R> Function<(A,)> for F
where
    F: Fn(A) -> R,
{
    type Result = R;

    fn invoke(&self, arg: (A,)) -> Self::Result {
        (self)(arg.0)
    }
}

impl<A, B, F, R> Function<(A, B)> for F
where
    F: Fn(A, B) -> R,
{
    type Result = R;

    fn invoke(&self, args: (A, B)) -> Self::Result {
        (self)(args.0, args.1)
    }
}

/// Similar to a `Function` but also takes an explicit `receiver`
/// parameter than is the first argument of the call (i.e. the `self` param);
pub trait Method<Receiver, Args = ()> {
    type Result;

    fn invoke(&self, receiver: &Receiver, args: Args) -> Self::Result;
}

impl<F, R, Receiver> Method<Receiver, ()> for F
where
    F: Fn(&Receiver) -> R,
{
    type Result = R;

    fn invoke(&self, receiver: &Receiver, _: ()) -> Self::Result {
        (self)(receiver)
    }
}

impl<A, F, R, Receiver> Method<Receiver, (A,)> for F
where
    F: Fn(&Receiver, A) -> R,
{
    type Result = R;

    fn invoke(&self, receiver: &Receiver, arg: (A,)) -> Self::Result {
        (self)(receiver, arg.0)
    }
}

impl<A, B, F, R, Receiver> Method<Receiver, (A, B)> for F
where
    F: Fn(&Receiver, A, B) -> R,
{
    type Result = R;

    fn invoke(&self, receiver: &Receiver, args: (A, B)) -> Self::Result {
        (self)(receiver, args.0, args.1)
    }
}

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

    pub fn from_class_method(name: Name) -> Self {
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
