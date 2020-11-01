use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::mem::discriminant;

use dyn_clone::DynClone;

pub trait Monad<T>: DynClone {
    // unit / of
    fn new(item: T) -> Self
    where
        Self: Sized;

    // and_then / fold
    fn join(self, other: Self) -> Self
    where
        Self: Sized;

    // flat_map
    fn map(self, f: Box<dyn FnOnce(&Self) -> Self>) -> Self
    where
        Self: Sized;
}

dyn_clone::clone_trait_object!(<T> Monad<T>);

impl<T: Debug> Debug for Box<dyn Monad<T>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Box<dyn Monad>")
    }
}

impl<T> PartialEq for Box<dyn Monad<T>> {
    fn eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl<T: Eq> Eq for Box<dyn Monad<T>> {}

impl<T: Hash> Hash for Box<dyn Monad<T>> {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        discriminant(self).hash(state)
    }
}

#[cfg(test)]
mod test {
    use super::Monad;

    #[test]
    fn identity_monad() {
        #[derive(Clone, Debug, Eq, PartialEq)]
        struct Id(u8);

        impl Monad<u8> for Id {
            fn new(item: u8) -> Self {
                Self(item)
            }

            fn join(self, other: Self) -> Self {
                other
            }

            fn map(self, f: Box<dyn FnOnce(&Self) -> Self>) -> Self {
                f(&self)
            }
        }

        assert_eq!(Id::new(1), Id(1));
        assert_eq!(Id::new(1).join(Id::new(2)), Id(2));
        assert_eq!(Id::new(1).map(Box::new(|Id(i)| Id::new(i + 1))), Id::new(2));
    }
}
