#![allow(clippy::type_complexity)]
//! Traits to help with passing around methods of arbitrary arities
//! and to help downcast+convert the arguments.

/// An alternate version of the `Fn` trait
/// which encodes the types of the arguments
/// in a single type - a tuple.
pub trait Function<Args = ()>: Send + Sync + 'static {
    type Result: 'static;

    fn invoke(&self, args: Args) -> Self::Result;
}

/// Similar to a `Function` but also takes an explicit `receiver`
/// parameter than is the first argument of the call (i.e. the `self` param);
pub trait Method<Receiver, Args = ()>: Send + Sync + 'static {
    type Result: 'static;

    fn invoke(&self, receiver: &Receiver, args: Args) -> Self::Result;
}

// Generated Impls (see test)

impl<F, R: 'static> Function<()> for F
where
    F: Fn() -> R + Send + Sync + 'static,
{
    type Result = R;

    fn invoke(&self, _args: ()) -> Self::Result {
        (self)()
    }
}

impl<A, F, R: 'static> Function<(A,)> for F
where
    F: Fn(A) -> R + Send + Sync + 'static,
{
    type Result = R;

    fn invoke(&self, args: (A,)) -> Self::Result {
        (self)(args.0)
    }
}

impl<A, B, F, R: 'static> Function<(A, B)> for F
where
    F: Fn(A, B) -> R + Send + Sync + 'static,
{
    type Result = R;

    fn invoke(&self, args: (A, B)) -> Self::Result {
        (self)(args.0, args.1)
    }
}

impl<A, B, C, F, R: 'static> Function<(A, B, C)> for F
where
    F: Fn(A, B, C) -> R + Send + Sync + 'static,
{
    type Result = R;

    fn invoke(&self, args: (A, B, C)) -> Self::Result {
        (self)(args.0, args.1, args.2)
    }
}

impl<A, B, C, D, F, R: 'static> Function<(A, B, C, D)> for F
where
    F: Fn(A, B, C, D) -> R + Send + Sync + 'static,
{
    type Result = R;

    fn invoke(&self, args: (A, B, C, D)) -> Self::Result {
        (self)(args.0, args.1, args.2, args.3)
    }
}

impl<A, B, C, D, E, F, R: 'static> Function<(A, B, C, D, E)> for F
where
    F: Fn(A, B, C, D, E) -> R + Send + Sync + 'static,
{
    type Result = R;

    fn invoke(&self, args: (A, B, C, D, E)) -> Self::Result {
        (self)(args.0, args.1, args.2, args.3, args.4)
    }
}

impl<A, B, C, D, E, G, F, R: 'static> Function<(A, B, C, D, E, G)> for F
where
    F: Fn(A, B, C, D, E, G) -> R + Send + Sync + 'static,
{
    type Result = R;

    fn invoke(&self, args: (A, B, C, D, E, G)) -> Self::Result {
        (self)(args.0, args.1, args.2, args.3, args.4, args.5)
    }
}

impl<A, B, C, D, E, G, H, F, R: 'static> Function<(A, B, C, D, E, G, H)> for F
where
    F: Fn(A, B, C, D, E, G, H) -> R + Send + Sync + 'static,
{
    type Result = R;

    fn invoke(&self, args: (A, B, C, D, E, G, H)) -> Self::Result {
        (self)(args.0, args.1, args.2, args.3, args.4, args.5, args.6)
    }
}

impl<A, B, C, D, E, G, H, I, F, R: 'static> Function<(A, B, C, D, E, G, H, I)> for F
where
    F: Fn(A, B, C, D, E, G, H, I) -> R + Send + Sync + 'static,
{
    type Result = R;

    fn invoke(&self, args: (A, B, C, D, E, G, H, I)) -> Self::Result {
        (self)(
            args.0, args.1, args.2, args.3, args.4, args.5, args.6, args.7,
        )
    }
}

impl<A, B, C, D, E, G, H, I, J, F, R: 'static> Function<(A, B, C, D, E, G, H, I, J)> for F
where
    F: Fn(A, B, C, D, E, G, H, I, J) -> R + Send + Sync + 'static,
{
    type Result = R;

    fn invoke(&self, args: (A, B, C, D, E, G, H, I, J)) -> Self::Result {
        (self)(
            args.0, args.1, args.2, args.3, args.4, args.5, args.6, args.7, args.8,
        )
    }
}

impl<A, B, C, D, E, G, H, I, J, K, F, R: 'static> Function<(A, B, C, D, E, G, H, I, J, K)> for F
where
    F: Fn(A, B, C, D, E, G, H, I, J, K) -> R + Send + Sync + 'static,
{
    type Result = R;

    fn invoke(&self, args: (A, B, C, D, E, G, H, I, J, K)) -> Self::Result {
        (self)(
            args.0, args.1, args.2, args.3, args.4, args.5, args.6, args.7, args.8, args.9,
        )
    }
}

impl<A, B, C, D, E, G, H, I, J, K, L, F, R: 'static> Function<(A, B, C, D, E, G, H, I, J, K, L)>
    for F
where
    F: Fn(A, B, C, D, E, G, H, I, J, K, L) -> R + Send + Sync + 'static,
{
    type Result = R;

    fn invoke(&self, args: (A, B, C, D, E, G, H, I, J, K, L)) -> Self::Result {
        (self)(
            args.0, args.1, args.2, args.3, args.4, args.5, args.6, args.7, args.8, args.9, args.10,
        )
    }
}

impl<A, B, C, D, E, G, H, I, J, K, L, M, F, R: 'static>
    Function<(A, B, C, D, E, G, H, I, J, K, L, M)> for F
where
    F: Fn(A, B, C, D, E, G, H, I, J, K, L, M) -> R + Send + Sync + 'static,
{
    type Result = R;

    fn invoke(&self, args: (A, B, C, D, E, G, H, I, J, K, L, M)) -> Self::Result {
        (self)(
            args.0, args.1, args.2, args.3, args.4, args.5, args.6, args.7, args.8, args.9,
            args.10, args.11,
        )
    }
}

impl<A, B, C, D, E, G, H, I, J, K, L, M, N, F, R: 'static>
    Function<(A, B, C, D, E, G, H, I, J, K, L, M, N)> for F
where
    F: Fn(A, B, C, D, E, G, H, I, J, K, L, M, N) -> R + Send + Sync + 'static,
{
    type Result = R;

    fn invoke(&self, args: (A, B, C, D, E, G, H, I, J, K, L, M, N)) -> Self::Result {
        (self)(
            args.0, args.1, args.2, args.3, args.4, args.5, args.6, args.7, args.8, args.9,
            args.10, args.11, args.12,
        )
    }
}

impl<A, B, C, D, E, G, H, I, J, K, L, M, N, O, F, R: 'static>
    Function<(A, B, C, D, E, G, H, I, J, K, L, M, N, O)> for F
where
    F: Fn(A, B, C, D, E, G, H, I, J, K, L, M, N, O) -> R + Send + Sync + 'static,
{
    type Result = R;

    fn invoke(&self, args: (A, B, C, D, E, G, H, I, J, K, L, M, N, O)) -> Self::Result {
        (self)(
            args.0, args.1, args.2, args.3, args.4, args.5, args.6, args.7, args.8, args.9,
            args.10, args.11, args.12, args.13,
        )
    }
}

impl<A, B, C, D, E, G, H, I, J, K, L, M, N, O, P, F, R: 'static>
    Function<(A, B, C, D, E, G, H, I, J, K, L, M, N, O, P)> for F
where
    F: Fn(A, B, C, D, E, G, H, I, J, K, L, M, N, O, P) -> R + Send + Sync + 'static,
{
    type Result = R;

    fn invoke(&self, args: (A, B, C, D, E, G, H, I, J, K, L, M, N, O, P)) -> Self::Result {
        (self)(
            args.0, args.1, args.2, args.3, args.4, args.5, args.6, args.7, args.8, args.9,
            args.10, args.11, args.12, args.13, args.14,
        )
    }
}

impl<A, B, C, D, E, G, H, I, J, K, L, M, N, O, P, Q, F, R: 'static>
    Function<(A, B, C, D, E, G, H, I, J, K, L, M, N, O, P, Q)> for F
where
    F: Fn(A, B, C, D, E, G, H, I, J, K, L, M, N, O, P, Q) -> R + Send + Sync + 'static,
{
    type Result = R;

    fn invoke(&self, args: (A, B, C, D, E, G, H, I, J, K, L, M, N, O, P, Q)) -> Self::Result {
        (self)(
            args.0, args.1, args.2, args.3, args.4, args.5, args.6, args.7, args.8, args.9,
            args.10, args.11, args.12, args.13, args.14, args.15,
        )
    }
}

impl<F, R: 'static, Receiver> Method<Receiver, ()> for F
where
    F: Fn(&Receiver) -> R + Send + Sync + 'static,
{
    type Result = R;

    fn invoke(&self, receiver: &Receiver, _args: ()) -> Self::Result {
        (self)(receiver)
    }
}

impl<A, F, R: 'static, Receiver> Method<Receiver, (A,)> for F
where
    F: Fn(&Receiver, A) -> R + Send + Sync + 'static,
{
    type Result = R;

    fn invoke(&self, receiver: &Receiver, args: (A,)) -> Self::Result {
        (self)(receiver, args.0)
    }
}

impl<A, B, F, R: 'static, Receiver> Method<Receiver, (A, B)> for F
where
    F: Fn(&Receiver, A, B) -> R + Send + Sync + 'static,
{
    type Result = R;

    fn invoke(&self, receiver: &Receiver, args: (A, B)) -> Self::Result {
        (self)(receiver, args.0, args.1)
    }
}

impl<A, B, C, F, R: 'static, Receiver> Method<Receiver, (A, B, C)> for F
where
    F: Fn(&Receiver, A, B, C) -> R + Send + Sync + 'static,
{
    type Result = R;

    fn invoke(&self, receiver: &Receiver, args: (A, B, C)) -> Self::Result {
        (self)(receiver, args.0, args.1, args.2)
    }
}

impl<A, B, C, D, F, R: 'static, Receiver> Method<Receiver, (A, B, C, D)> for F
where
    F: Fn(&Receiver, A, B, C, D) -> R + Send + Sync + 'static,
{
    type Result = R;

    fn invoke(&self, receiver: &Receiver, args: (A, B, C, D)) -> Self::Result {
        (self)(receiver, args.0, args.1, args.2, args.3)
    }
}

impl<A, B, C, D, E, F, R: 'static, Receiver> Method<Receiver, (A, B, C, D, E)> for F
where
    F: Fn(&Receiver, A, B, C, D, E) -> R + Send + Sync + 'static,
{
    type Result = R;

    fn invoke(&self, receiver: &Receiver, args: (A, B, C, D, E)) -> Self::Result {
        (self)(receiver, args.0, args.1, args.2, args.3, args.4)
    }
}

impl<A, B, C, D, E, G, F, R: 'static, Receiver> Method<Receiver, (A, B, C, D, E, G)> for F
where
    F: Fn(&Receiver, A, B, C, D, E, G) -> R + Send + Sync + 'static,
{
    type Result = R;

    fn invoke(&self, receiver: &Receiver, args: (A, B, C, D, E, G)) -> Self::Result {
        (self)(receiver, args.0, args.1, args.2, args.3, args.4, args.5)
    }
}

impl<A, B, C, D, E, G, H, F, R: 'static, Receiver> Method<Receiver, (A, B, C, D, E, G, H)> for F
where
    F: Fn(&Receiver, A, B, C, D, E, G, H) -> R + Send + Sync + 'static,
{
    type Result = R;

    fn invoke(&self, receiver: &Receiver, args: (A, B, C, D, E, G, H)) -> Self::Result {
        (self)(
            receiver, args.0, args.1, args.2, args.3, args.4, args.5, args.6,
        )
    }
}

impl<A, B, C, D, E, G, H, I, F, R: 'static, Receiver> Method<Receiver, (A, B, C, D, E, G, H, I)>
    for F
where
    F: Fn(&Receiver, A, B, C, D, E, G, H, I) -> R + Send + Sync + 'static,
{
    type Result = R;

    fn invoke(&self, receiver: &Receiver, args: (A, B, C, D, E, G, H, I)) -> Self::Result {
        (self)(
            receiver, args.0, args.1, args.2, args.3, args.4, args.5, args.6, args.7,
        )
    }
}

impl<A, B, C, D, E, G, H, I, J, F, R: 'static, Receiver>
    Method<Receiver, (A, B, C, D, E, G, H, I, J)> for F
where
    F: Fn(&Receiver, A, B, C, D, E, G, H, I, J) -> R + Send + Sync + 'static,
{
    type Result = R;

    fn invoke(&self, receiver: &Receiver, args: (A, B, C, D, E, G, H, I, J)) -> Self::Result {
        (self)(
            receiver, args.0, args.1, args.2, args.3, args.4, args.5, args.6, args.7, args.8,
        )
    }
}

impl<A, B, C, D, E, G, H, I, J, K, F, R: 'static, Receiver>
    Method<Receiver, (A, B, C, D, E, G, H, I, J, K)> for F
where
    F: Fn(&Receiver, A, B, C, D, E, G, H, I, J, K) -> R + Send + Sync + 'static,
{
    type Result = R;

    fn invoke(&self, receiver: &Receiver, args: (A, B, C, D, E, G, H, I, J, K)) -> Self::Result {
        (self)(
            receiver, args.0, args.1, args.2, args.3, args.4, args.5, args.6, args.7, args.8,
            args.9,
        )
    }
}

impl<A, B, C, D, E, G, H, I, J, K, L, F, R: 'static, Receiver>
    Method<Receiver, (A, B, C, D, E, G, H, I, J, K, L)> for F
where
    F: Fn(&Receiver, A, B, C, D, E, G, H, I, J, K, L) -> R + Send + Sync + 'static,
{
    type Result = R;

    fn invoke(&self, receiver: &Receiver, args: (A, B, C, D, E, G, H, I, J, K, L)) -> Self::Result {
        (self)(
            receiver, args.0, args.1, args.2, args.3, args.4, args.5, args.6, args.7, args.8,
            args.9, args.10,
        )
    }
}

impl<A, B, C, D, E, G, H, I, J, K, L, M, F, R: 'static, Receiver>
    Method<Receiver, (A, B, C, D, E, G, H, I, J, K, L, M)> for F
where
    F: Fn(&Receiver, A, B, C, D, E, G, H, I, J, K, L, M) -> R + Send + Sync + 'static,
{
    type Result = R;

    fn invoke(
        &self,
        receiver: &Receiver,
        args: (A, B, C, D, E, G, H, I, J, K, L, M),
    ) -> Self::Result {
        (self)(
            receiver, args.0, args.1, args.2, args.3, args.4, args.5, args.6, args.7, args.8,
            args.9, args.10, args.11,
        )
    }
}

impl<A, B, C, D, E, G, H, I, J, K, L, M, N, F, R: 'static, Receiver>
    Method<Receiver, (A, B, C, D, E, G, H, I, J, K, L, M, N)> for F
where
    F: Fn(&Receiver, A, B, C, D, E, G, H, I, J, K, L, M, N) -> R + Send + Sync + 'static,
{
    type Result = R;

    fn invoke(
        &self,
        receiver: &Receiver,
        args: (A, B, C, D, E, G, H, I, J, K, L, M, N),
    ) -> Self::Result {
        (self)(
            receiver, args.0, args.1, args.2, args.3, args.4, args.5, args.6, args.7, args.8,
            args.9, args.10, args.11, args.12,
        )
    }
}

impl<A, B, C, D, E, G, H, I, J, K, L, M, N, O, F, R: 'static, Receiver>
    Method<Receiver, (A, B, C, D, E, G, H, I, J, K, L, M, N, O)> for F
where
    F: Fn(&Receiver, A, B, C, D, E, G, H, I, J, K, L, M, N, O) -> R + Send + Sync + 'static,
{
    type Result = R;

    fn invoke(
        &self,
        receiver: &Receiver,
        args: (A, B, C, D, E, G, H, I, J, K, L, M, N, O),
    ) -> Self::Result {
        (self)(
            receiver, args.0, args.1, args.2, args.3, args.4, args.5, args.6, args.7, args.8,
            args.9, args.10, args.11, args.12, args.13,
        )
    }
}

impl<A, B, C, D, E, G, H, I, J, K, L, M, N, O, P, F, R: 'static, Receiver>
    Method<Receiver, (A, B, C, D, E, G, H, I, J, K, L, M, N, O, P)> for F
where
    F: Fn(&Receiver, A, B, C, D, E, G, H, I, J, K, L, M, N, O, P) -> R + Send + Sync + 'static,
{
    type Result = R;

    fn invoke(
        &self,
        receiver: &Receiver,
        args: (A, B, C, D, E, G, H, I, J, K, L, M, N, O, P),
    ) -> Self::Result {
        (self)(
            receiver, args.0, args.1, args.2, args.3, args.4, args.5, args.6, args.7, args.8,
            args.9, args.10, args.11, args.12, args.13, args.14,
        )
    }
}

impl<A, B, C, D, E, G, H, I, J, K, L, M, N, O, P, Q, F, R: 'static, Receiver>
    Method<Receiver, (A, B, C, D, E, G, H, I, J, K, L, M, N, O, P, Q)> for F
where
    F: Fn(&Receiver, A, B, C, D, E, G, H, I, J, K, L, M, N, O, P, Q) -> R + Send + Sync + 'static,
{
    type Result = R;

    fn invoke(
        &self,
        receiver: &Receiver,
        args: (A, B, C, D, E, G, H, I, J, K, L, M, N, O, P, Q),
    ) -> Self::Result {
        (self)(
            receiver, args.0, args.1, args.2, args.3, args.4, args.5, args.6, args.7, args.8,
            args.9, args.10, args.11, args.12, args.13, args.14, args.15,
        )
    }
}

#[cfg(test)]
mod tests {
    // This would be great if it was a proc macro but I don't wanna set up another crate.
    // @WOW Hack, run this test and paste in the input.
    #[ignore]
    #[test]
    fn gen_impls() {
        fn tuple_type(i: usize) -> (String, String) {
            let letters: [char; 16] = [
                'A', 'B', 'C', 'D', 'E', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q',
            ];
            assert!(i <= letters.len());
            match i {
                0 => ("".to_owned(), "".to_owned()),
                1 => ("A,".to_owned(), "args.0".to_owned()),
                n => {
                    let letters = letters
                        .iter()
                        .take(n)
                        .map(|i| i.to_string())
                        .collect::<Vec<String>>()
                        .join(", ");
                    let args = (0..i)
                        .map(|i| format!("args.{}", i))
                        .collect::<Vec<String>>()
                        .join(", ");
                    (format!("{}, ", letters), format!("({})", args))
                }
            }
        }

        eprintln!("// Generated Impls (see test)");
        for i in 0..=16 {
            let (letters, args) = tuple_type(i);
            let imp = format!(
                r#"
impl<{letters}F, R: 'static> Function<({letters})> for F
where
    F: Fn({letters}) -> R + Send + Sync + 'static,
{{
    type Result = R;

    fn invoke(&self, args: ({letters})) -> Self::Result {{
        (self)({args})
    }}
}}

impl<{letters}F, R: 'static, Receiver> Method<Receiver, ({letters})> for F
where
    F: Fn(&Receiver, {letters}) -> R + Send + Sync + 'static,
{{
    type Result = R;

    fn invoke(&self, receiver: &Receiver, args: ({letters})) -> Self::Result {{
        (self)(receiver, {args})
    }}
}}
"#,
                letters = letters,
                args = args
            );
            eprintln!("{}", imp);
        }
    }
}
