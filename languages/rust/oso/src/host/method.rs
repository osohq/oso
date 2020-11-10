//! Traits to help with passing around methods of arbitrary arities

/// An alternate version of the `Fn` trait
/// which encodes the types of the arguments
/// in a single type - a tuple.
pub trait Function<Args = ()>: Send + Sync + 'static {
    type Result;

    fn invoke(&self, args: Args) -> Self::Result;
}

/// Similar to a `Function` but also takes an explicit `receiver`
/// parameter than is the first argument of the call (i.e. the `self` param);
pub trait Method<Receiver, Args = ()>: Send + Sync + 'static {
    type Result;

    fn invoke(&self, receiver: &Receiver, args: Args) -> Self::Result;
}

macro_rules! tuple_impls {
    ( $( $name:ident )* ) => {
        impl<Fun, Res, $($name),*> Function<($($name,)*)> for Fun
        where
            Fun: Fn($($name),*) -> Res + Send + Sync + 'static
        {
            type Result = Res;

            fn invoke(&self, args: ($($name,)*)) -> Self::Result {
                #[allow(non_snake_case)]
                let ($($name,)*) = args;
                (self)($($name,)*)
            }
        }

        impl<Fun, Res, Receiver, $($name),*> Method<Receiver, ($($name,)*)> for Fun
        where
            Fun: Fn(&Receiver, $($name),*) -> Res + Send + Sync + 'static,
        {
            type Result = Res;

            fn invoke(&self, receiver: &Receiver, args: ($($name,)*)) -> Self::Result {
                #[allow(non_snake_case)]
                let ($($name,)*) = args;
                (self)(receiver, $($name,)*)
            }
        }
    };
}

tuple_impls! {}
tuple_impls! { A }
tuple_impls! { A B }
tuple_impls! { A B C }
tuple_impls! { A B C D }
tuple_impls! { A B C D E }
tuple_impls! { A B C D E F }
tuple_impls! { A B C D E F G }
tuple_impls! { A B C D E F G H }
tuple_impls! { A B C D E F G H I }
tuple_impls! { A B C D E F G H I J }
tuple_impls! { A B C D E F G H I J K }
tuple_impls! { A B C D E F G H I J K L }
tuple_impls! { A B C D E F G H I J K L M }
tuple_impls! { A B C D E F G H I J K L M N }
tuple_impls! { A B C D E F G H I J K L M N O }
tuple_impls! { A B C D E F G H I J K L M N O P }
