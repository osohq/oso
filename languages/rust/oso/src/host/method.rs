//! Traits to help with passing around methods of arbitrary arities
//! and to help downcast+convert the arguments.

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
