#![allow(clippy::many_single_char_names, clippy::type_complexity)]
//! Trait and implementations of `FromPolar` for converting from
//! Polar types back to Rust types.

use polar_core::terms::*;

use std::collections::HashMap;
use std::convert::TryFrom;

use super::class::Instance;
use super::{Host, HostClass};

pub trait FromPolar: Sized {
    fn from_polar(term: &Term, host: &mut Host) -> crate::Result<Self>;

    fn from_polar_list(terms: &[Term], host: &mut Host) -> crate::Result<Self> {
        assert_eq!(terms.len(), 1);
        Self::from_polar(&terms[0], host)
    }
}

impl<C: 'static + Clone + HostClass> FromPolar for C {
    fn from_polar(term: &Term, host: &mut Host) -> crate::Result<Self> {
        match term.value() {
            Value::ExternalInstance(ExternalInstance { instance_id, .. }) => host
                .get_instance(*instance_id)
                .and_then(|instance| instance.instance.downcast_ref::<C>().cloned())
                .ok_or_else(|| crate::OsoError::FromPolar),
            _ => Err(crate::OsoError::FromPolar),
        }
    }
}

impl FromPolar for bool {
    fn from_polar(term: &Term, _host: &mut Host) -> crate::Result<Self> {
        if let Value::Boolean(b) = term.value() {
            Ok(*b)
        } else {
            Err(crate::OsoError::FromPolar)
        }
    }
}

macro_rules! polar_to_int {
    ($i:ty) => {
        impl FromPolar for $i {
            fn from_polar(term: &Term, _host: &mut Host) -> crate::Result<Self> {
                if let Value::Number(Numeric::Integer(i)) = term.value() {
                    <$i>::try_from(*i).map_err(|_| crate::OsoError::FromPolar)
                } else {
                    Err(crate::OsoError::FromPolar)
                }
            }
        }
    };
}

polar_to_int!(u8);
polar_to_int!(i8);
polar_to_int!(u16);
polar_to_int!(i16);
polar_to_int!(u32);
polar_to_int!(i32);
polar_to_int!(i64);

impl FromPolar for f64 {
    fn from_polar(term: &Term, _host: &mut Host) -> crate::Result<Self> {
        if let Value::Number(Numeric::Float(f)) = term.value() {
            Ok(*f)
        } else {
            Err(crate::OsoError::FromPolar)
        }
    }
}

impl FromPolar for String {
    fn from_polar(term: &Term, _host: &mut Host) -> crate::Result<Self> {
        if let Value::String(s) = term.value() {
            Ok(s.to_string())
        } else {
            Err(crate::OsoError::FromPolar)
        }
    }
}

impl<T: FromPolar> FromPolar for Vec<T> {
    fn from_polar(term: &Term, host: &mut Host) -> crate::Result<Self> {
        if let Value::List(l) = term.value() {
            l.iter().map(|t| T::from_polar(t, host)).collect()
        } else {
            Err(crate::OsoError::FromPolar)
        }
    }

    fn from_polar_list(terms: &[Term], host: &mut Host) -> crate::Result<Self> {
        terms.iter().map(|t| T::from_polar(t, host)).collect()
    }
}

impl<T: FromPolar> FromPolar for HashMap<String, T> {
    fn from_polar(term: &Term, host: &mut Host) -> crate::Result<Self> {
        if let Value::Dictionary(dict) = term.value() {
            dict.fields
                .iter()
                .map(|(k, v)| T::from_polar(v, host).map(|v| (k.0.clone(), v)))
                .collect()
        } else {
            Err(crate::OsoError::FromPolar)
        }
    }
}

impl FromPolar for Value {
    fn from_polar(term: &Term, _host: &mut Host) -> crate::Result<Self> {
        Ok(term.value().clone())
    }
}

impl FromPolar for Instance {
    fn from_polar(term: &Term, host: &mut Host) -> crate::Result<Self> {
        //  TODO (dhatch): Why do we have cases for anything besides external instance?
        let instance = match term.value().clone() {
            Value::Boolean(b) => host
                .get_class_from_type::<bool>()
                .unwrap()
                .cast_to_instance(b),
            Value::Number(Numeric::Integer(i)) => host
                .get_class_from_type::<i64>()
                .unwrap()
                .cast_to_instance(i),
            Value::Number(Numeric::Float(f)) => host
                .get_class_from_type::<f64>()
                .unwrap()
                .cast_to_instance(f),
            Value::List(v) => host
                .get_class_from_type::<Vec<Term>>()
                .unwrap()
                .cast_to_instance(v),
            Value::String(s) => host
                .get_class_from_type::<String>()
                .unwrap()
                .cast_to_instance(s),
            Value::Dictionary(d) => host
                .get_class_from_type::<HashMap<Symbol, Term>>()
                .unwrap()
                .cast_to_instance(d.fields),
            Value::ExternalInstance(ExternalInstance { instance_id, .. }) => host
                .get_instance(instance_id)
                .expect("instance not found")
                .clone(),
            v => {
                tracing::warn!(value = ?v, "invalid conversion attempted");
                return Err(crate::OsoError::FromPolar);
            }
        };
        Ok(instance)
    }
}

impl FromPolar for () {
    fn from_polar(term: &Term, _host: &mut Host) -> crate::Result<Self> {
        if let Value::List(l) = term.value() {
            if l.is_empty() {
                Ok(())
            } else {
                Err(crate::OsoError::FromPolar)
            }
        } else {
            Err(crate::OsoError::FromPolar)
        }
    }

    fn from_polar_list(terms: &[Term], _host: &mut Host) -> crate::Result<Self> {
        if terms.is_empty() {
            Ok(())
        } else {
            Err(crate::OsoError::FromPolar)
        }
    }
}

impl<A> FromPolar for (A,)
where
    A: FromPolar,
{
    fn from_polar(_term: &Term, _host: &mut Host) -> crate::Result<Self> {
        Err(crate::OsoError::FromPolar)
    }

    fn from_polar_list(terms: &[Term], host: &mut Host) -> crate::Result<Self> {
        if terms.len() == 1 {
            A::from_polar(&terms[0], host).map(|a| (a,))
        } else {
            Err(crate::OsoError::FromPolar)
        }
    }
}

impl<A, B> FromPolar for (A, B)
where
    A: FromPolar,
    B: FromPolar,
{
    fn from_polar(_term: &Term, _host: &mut Host) -> crate::Result<Self> {
        Err(crate::OsoError::FromPolar)
    }

    fn from_polar_list(terms: &[Term], host: &mut Host) -> crate::Result<Self> {
        if terms.len() == 2 {
            let a = A::from_polar(&terms[0], host)?;
            let b = B::from_polar(&terms[1], host)?;
            Ok((a, b))
        } else {
            Err(crate::OsoError::FromPolar)
        }
    }
}
impl<A, B, C> FromPolar for (A, B, C)
where
    A: FromPolar,
    B: FromPolar,
    C: FromPolar,
{
    fn from_polar(_term: &Term, _host: &mut Host) -> crate::Result<Self> {
        Err(crate::OsoError::FromPolar)
    }

    fn from_polar_list(terms: &[Term], host: &mut Host) -> crate::Result<Self> {
        if terms.len() == 3 {
            let a = A::from_polar(&terms[0], host)?;
            let b = B::from_polar(&terms[1], host)?;
            let c = C::from_polar(&terms[2], host)?;
            Ok((a, b, c))
        } else {
            Err(crate::OsoError::FromPolar)
        }
    }
}
impl<A, B, C, D> FromPolar for (A, B, C, D)
where
    A: FromPolar,
    B: FromPolar,
    C: FromPolar,
    D: FromPolar,
{
    fn from_polar(_term: &Term, _host: &mut Host) -> crate::Result<Self> {
        Err(crate::OsoError::FromPolar)
    }

    fn from_polar_list(terms: &[Term], host: &mut Host) -> crate::Result<Self> {
        if terms.len() == 4 {
            let a = A::from_polar(&terms[0], host)?;
            let b = B::from_polar(&terms[1], host)?;
            let c = C::from_polar(&terms[2], host)?;
            let d = D::from_polar(&terms[3], host)?;
            Ok((a, b, c, d))
        } else {
            Err(crate::OsoError::FromPolar)
        }
    }
}
impl<A, B, C, D, E> FromPolar for (A, B, C, D, E)
where
    A: FromPolar,
    B: FromPolar,
    C: FromPolar,
    D: FromPolar,
    E: FromPolar,
{
    fn from_polar(_term: &Term, _host: &mut Host) -> crate::Result<Self> {
        Err(crate::OsoError::FromPolar)
    }

    fn from_polar_list(terms: &[Term], host: &mut Host) -> crate::Result<Self> {
        if terms.len() == 5 {
            let a = A::from_polar(&terms[0], host)?;
            let b = B::from_polar(&terms[1], host)?;
            let c = C::from_polar(&terms[2], host)?;
            let d = D::from_polar(&terms[3], host)?;
            let e = E::from_polar(&terms[4], host)?;
            Ok((a, b, c, d, e))
        } else {
            Err(crate::OsoError::FromPolar)
        }
    }
}
impl<A, B, C, D, E, G> FromPolar for (A, B, C, D, E, G)
where
    A: FromPolar,
    B: FromPolar,
    C: FromPolar,
    D: FromPolar,
    E: FromPolar,
    G: FromPolar,
{
    fn from_polar(_term: &Term, _host: &mut Host) -> crate::Result<Self> {
        Err(crate::OsoError::FromPolar)
    }

    fn from_polar_list(terms: &[Term], host: &mut Host) -> crate::Result<Self> {
        if terms.len() == 6 {
            let a = A::from_polar(&terms[0], host)?;
            let b = B::from_polar(&terms[1], host)?;
            let c = C::from_polar(&terms[2], host)?;
            let d = D::from_polar(&terms[3], host)?;
            let e = E::from_polar(&terms[4], host)?;
            let g = G::from_polar(&terms[5], host)?;
            Ok((a, b, c, d, e, g))
        } else {
            Err(crate::OsoError::FromPolar)
        }
    }
}
impl<A, B, C, D, E, G, H> FromPolar for (A, B, C, D, E, G, H)
where
    A: FromPolar,
    B: FromPolar,
    C: FromPolar,
    D: FromPolar,
    E: FromPolar,
    G: FromPolar,
    H: FromPolar,
{
    fn from_polar(_term: &Term, _host: &mut Host) -> crate::Result<Self> {
        Err(crate::OsoError::FromPolar)
    }

    fn from_polar_list(terms: &[Term], host: &mut Host) -> crate::Result<Self> {
        if terms.len() == 7 {
            let a = A::from_polar(&terms[0], host)?;
            let b = B::from_polar(&terms[1], host)?;
            let c = C::from_polar(&terms[2], host)?;
            let d = D::from_polar(&terms[3], host)?;
            let e = E::from_polar(&terms[4], host)?;
            let g = G::from_polar(&terms[5], host)?;
            let h = H::from_polar(&terms[6], host)?;
            Ok((a, b, c, d, e, g, h))
        } else {
            Err(crate::OsoError::FromPolar)
        }
    }
}
impl<A, B, C, D, E, G, H, I> FromPolar for (A, B, C, D, E, G, H, I)
where
    A: FromPolar,
    B: FromPolar,
    C: FromPolar,
    D: FromPolar,
    E: FromPolar,
    G: FromPolar,
    H: FromPolar,
    I: FromPolar,
{
    fn from_polar(_term: &Term, _host: &mut Host) -> crate::Result<Self> {
        Err(crate::OsoError::FromPolar)
    }

    fn from_polar_list(terms: &[Term], host: &mut Host) -> crate::Result<Self> {
        if terms.len() == 8 {
            let a = A::from_polar(&terms[0], host)?;
            let b = B::from_polar(&terms[1], host)?;
            let c = C::from_polar(&terms[2], host)?;
            let d = D::from_polar(&terms[3], host)?;
            let e = E::from_polar(&terms[4], host)?;
            let g = G::from_polar(&terms[5], host)?;
            let h = H::from_polar(&terms[6], host)?;
            let i = I::from_polar(&terms[7], host)?;
            Ok((a, b, c, d, e, g, h, i))
        } else {
            Err(crate::OsoError::FromPolar)
        }
    }
}
impl<A, B, C, D, E, G, H, I, J> FromPolar for (A, B, C, D, E, G, H, I, J)
where
    A: FromPolar,
    B: FromPolar,
    C: FromPolar,
    D: FromPolar,
    E: FromPolar,
    G: FromPolar,
    H: FromPolar,
    I: FromPolar,
    J: FromPolar,
{
    fn from_polar(_term: &Term, _host: &mut Host) -> crate::Result<Self> {
        Err(crate::OsoError::FromPolar)
    }

    fn from_polar_list(terms: &[Term], host: &mut Host) -> crate::Result<Self> {
        if terms.len() == 9 {
            let a = A::from_polar(&terms[0], host)?;
            let b = B::from_polar(&terms[1], host)?;
            let c = C::from_polar(&terms[2], host)?;
            let d = D::from_polar(&terms[3], host)?;
            let e = E::from_polar(&terms[4], host)?;
            let g = G::from_polar(&terms[5], host)?;
            let h = H::from_polar(&terms[6], host)?;
            let i = I::from_polar(&terms[7], host)?;
            let j = J::from_polar(&terms[8], host)?;
            Ok((a, b, c, d, e, g, h, i, j))
        } else {
            Err(crate::OsoError::FromPolar)
        }
    }
}
impl<A, B, C, D, E, G, H, I, J, K> FromPolar for (A, B, C, D, E, G, H, I, J, K)
where
    A: FromPolar,
    B: FromPolar,
    C: FromPolar,
    D: FromPolar,
    E: FromPolar,
    G: FromPolar,
    H: FromPolar,
    I: FromPolar,
    J: FromPolar,
    K: FromPolar,
{
    fn from_polar(_term: &Term, _host: &mut Host) -> crate::Result<Self> {
        Err(crate::OsoError::FromPolar)
    }

    fn from_polar_list(terms: &[Term], host: &mut Host) -> crate::Result<Self> {
        if terms.len() == 10 {
            let a = A::from_polar(&terms[0], host)?;
            let b = B::from_polar(&terms[1], host)?;
            let c = C::from_polar(&terms[2], host)?;
            let d = D::from_polar(&terms[3], host)?;
            let e = E::from_polar(&terms[4], host)?;
            let g = G::from_polar(&terms[5], host)?;
            let h = H::from_polar(&terms[6], host)?;
            let i = I::from_polar(&terms[7], host)?;
            let j = J::from_polar(&terms[8], host)?;
            let k = K::from_polar(&terms[9], host)?;
            Ok((a, b, c, d, e, g, h, i, j, k))
        } else {
            Err(crate::OsoError::FromPolar)
        }
    }
}
impl<A, B, C, D, E, G, H, I, J, K, L> FromPolar for (A, B, C, D, E, G, H, I, J, K, L)
where
    A: FromPolar,
    B: FromPolar,
    C: FromPolar,
    D: FromPolar,
    E: FromPolar,
    G: FromPolar,
    H: FromPolar,
    I: FromPolar,
    J: FromPolar,
    K: FromPolar,
    L: FromPolar,
{
    fn from_polar(_term: &Term, _host: &mut Host) -> crate::Result<Self> {
        Err(crate::OsoError::FromPolar)
    }

    fn from_polar_list(terms: &[Term], host: &mut Host) -> crate::Result<Self> {
        if terms.len() == 11 {
            let a = A::from_polar(&terms[0], host)?;
            let b = B::from_polar(&terms[1], host)?;
            let c = C::from_polar(&terms[2], host)?;
            let d = D::from_polar(&terms[3], host)?;
            let e = E::from_polar(&terms[4], host)?;
            let g = G::from_polar(&terms[5], host)?;
            let h = H::from_polar(&terms[6], host)?;
            let i = I::from_polar(&terms[7], host)?;
            let j = J::from_polar(&terms[8], host)?;
            let k = K::from_polar(&terms[9], host)?;
            let l = L::from_polar(&terms[10], host)?;
            Ok((a, b, c, d, e, g, h, i, j, k, l))
        } else {
            Err(crate::OsoError::FromPolar)
        }
    }
}
impl<A, B, C, D, E, G, H, I, J, K, L, M> FromPolar for (A, B, C, D, E, G, H, I, J, K, L, M)
where
    A: FromPolar,
    B: FromPolar,
    C: FromPolar,
    D: FromPolar,
    E: FromPolar,
    G: FromPolar,
    H: FromPolar,
    I: FromPolar,
    J: FromPolar,
    K: FromPolar,
    L: FromPolar,
    M: FromPolar,
{
    fn from_polar(_term: &Term, _host: &mut Host) -> crate::Result<Self> {
        Err(crate::OsoError::FromPolar)
    }

    fn from_polar_list(terms: &[Term], host: &mut Host) -> crate::Result<Self> {
        if terms.len() == 12 {
            let a = A::from_polar(&terms[0], host)?;
            let b = B::from_polar(&terms[1], host)?;
            let c = C::from_polar(&terms[2], host)?;
            let d = D::from_polar(&terms[3], host)?;
            let e = E::from_polar(&terms[4], host)?;
            let g = G::from_polar(&terms[5], host)?;
            let h = H::from_polar(&terms[6], host)?;
            let i = I::from_polar(&terms[7], host)?;
            let j = J::from_polar(&terms[8], host)?;
            let k = K::from_polar(&terms[9], host)?;
            let l = L::from_polar(&terms[10], host)?;
            let m = M::from_polar(&terms[11], host)?;
            Ok((a, b, c, d, e, g, h, i, j, k, l, m))
        } else {
            Err(crate::OsoError::FromPolar)
        }
    }
}
impl<A, B, C, D, E, G, H, I, J, K, L, M, N> FromPolar for (A, B, C, D, E, G, H, I, J, K, L, M, N)
where
    A: FromPolar,
    B: FromPolar,
    C: FromPolar,
    D: FromPolar,
    E: FromPolar,
    G: FromPolar,
    H: FromPolar,
    I: FromPolar,
    J: FromPolar,
    K: FromPolar,
    L: FromPolar,
    M: FromPolar,
    N: FromPolar,
{
    fn from_polar(_term: &Term, _host: &mut Host) -> crate::Result<Self> {
        Err(crate::OsoError::FromPolar)
    }

    fn from_polar_list(terms: &[Term], host: &mut Host) -> crate::Result<Self> {
        if terms.len() == 13 {
            let a = A::from_polar(&terms[0], host)?;
            let b = B::from_polar(&terms[1], host)?;
            let c = C::from_polar(&terms[2], host)?;
            let d = D::from_polar(&terms[3], host)?;
            let e = E::from_polar(&terms[4], host)?;
            let g = G::from_polar(&terms[5], host)?;
            let h = H::from_polar(&terms[6], host)?;
            let i = I::from_polar(&terms[7], host)?;
            let j = J::from_polar(&terms[8], host)?;
            let k = K::from_polar(&terms[9], host)?;
            let l = L::from_polar(&terms[10], host)?;
            let m = M::from_polar(&terms[11], host)?;
            let n = N::from_polar(&terms[12], host)?;
            Ok((a, b, c, d, e, g, h, i, j, k, l, m, n))
        } else {
            Err(crate::OsoError::FromPolar)
        }
    }
}
impl<A, B, C, D, E, G, H, I, J, K, L, M, N, O> FromPolar
    for (A, B, C, D, E, G, H, I, J, K, L, M, N, O)
where
    A: FromPolar,
    B: FromPolar,
    C: FromPolar,
    D: FromPolar,
    E: FromPolar,
    G: FromPolar,
    H: FromPolar,
    I: FromPolar,
    J: FromPolar,
    K: FromPolar,
    L: FromPolar,
    M: FromPolar,
    N: FromPolar,
    O: FromPolar,
{
    fn from_polar(_term: &Term, _host: &mut Host) -> crate::Result<Self> {
        Err(crate::OsoError::FromPolar)
    }

    fn from_polar_list(terms: &[Term], host: &mut Host) -> crate::Result<Self> {
        if terms.len() == 14 {
            let a = A::from_polar(&terms[0], host)?;
            let b = B::from_polar(&terms[1], host)?;
            let c = C::from_polar(&terms[2], host)?;
            let d = D::from_polar(&terms[3], host)?;
            let e = E::from_polar(&terms[4], host)?;
            let g = G::from_polar(&terms[5], host)?;
            let h = H::from_polar(&terms[6], host)?;
            let i = I::from_polar(&terms[7], host)?;
            let j = J::from_polar(&terms[8], host)?;
            let k = K::from_polar(&terms[9], host)?;
            let l = L::from_polar(&terms[10], host)?;
            let m = M::from_polar(&terms[11], host)?;
            let n = N::from_polar(&terms[12], host)?;
            let o = O::from_polar(&terms[13], host)?;
            Ok((a, b, c, d, e, g, h, i, j, k, l, m, n, o))
        } else {
            Err(crate::OsoError::FromPolar)
        }
    }
}
impl<A, B, C, D, E, G, H, I, J, K, L, M, N, O, P> FromPolar
    for (A, B, C, D, E, G, H, I, J, K, L, M, N, O, P)
where
    A: FromPolar,
    B: FromPolar,
    C: FromPolar,
    D: FromPolar,
    E: FromPolar,
    G: FromPolar,
    H: FromPolar,
    I: FromPolar,
    J: FromPolar,
    K: FromPolar,
    L: FromPolar,
    M: FromPolar,
    N: FromPolar,
    O: FromPolar,
    P: FromPolar,
{
    fn from_polar(_term: &Term, _host: &mut Host) -> crate::Result<Self> {
        Err(crate::OsoError::FromPolar)
    }

    fn from_polar_list(terms: &[Term], host: &mut Host) -> crate::Result<Self> {
        if terms.len() == 15 {
            let a = A::from_polar(&terms[0], host)?;
            let b = B::from_polar(&terms[1], host)?;
            let c = C::from_polar(&terms[2], host)?;
            let d = D::from_polar(&terms[3], host)?;
            let e = E::from_polar(&terms[4], host)?;
            let g = G::from_polar(&terms[5], host)?;
            let h = H::from_polar(&terms[6], host)?;
            let i = I::from_polar(&terms[7], host)?;
            let j = J::from_polar(&terms[8], host)?;
            let k = K::from_polar(&terms[9], host)?;
            let l = L::from_polar(&terms[10], host)?;
            let m = M::from_polar(&terms[11], host)?;
            let n = N::from_polar(&terms[12], host)?;
            let o = O::from_polar(&terms[13], host)?;
            let p = P::from_polar(&terms[14], host)?;
            Ok((a, b, c, d, e, g, h, i, j, k, l, m, n, o, p))
        } else {
            Err(crate::OsoError::FromPolar)
        }
    }
}
impl<A, B, C, D, E, G, H, I, J, K, L, M, N, O, P, Q> FromPolar
    for (A, B, C, D, E, G, H, I, J, K, L, M, N, O, P, Q)
where
    A: FromPolar,
    B: FromPolar,
    C: FromPolar,
    D: FromPolar,
    E: FromPolar,
    G: FromPolar,
    H: FromPolar,
    I: FromPolar,
    J: FromPolar,
    K: FromPolar,
    L: FromPolar,
    M: FromPolar,
    N: FromPolar,
    O: FromPolar,
    P: FromPolar,
    Q: FromPolar,
{
    fn from_polar(_term: &Term, _host: &mut Host) -> crate::Result<Self> {
        Err(crate::OsoError::FromPolar)
    }

    fn from_polar_list(terms: &[Term], host: &mut Host) -> crate::Result<Self> {
        if terms.len() == 16 {
            let a = A::from_polar(&terms[0], host)?;
            let b = B::from_polar(&terms[1], host)?;
            let c = C::from_polar(&terms[2], host)?;
            let d = D::from_polar(&terms[3], host)?;
            let e = E::from_polar(&terms[4], host)?;
            let g = G::from_polar(&terms[5], host)?;
            let h = H::from_polar(&terms[6], host)?;
            let i = I::from_polar(&terms[7], host)?;
            let j = J::from_polar(&terms[8], host)?;
            let k = K::from_polar(&terms[9], host)?;
            let l = L::from_polar(&terms[10], host)?;
            let m = M::from_polar(&terms[11], host)?;
            let n = N::from_polar(&terms[12], host)?;
            let o = O::from_polar(&terms[13], host)?;
            let p = P::from_polar(&terms[14], host)?;
            let q = Q::from_polar(&terms[15], host)?;
            Ok((a, b, c, d, e, g, h, i, j, k, l, m, n, o, p, q))
        } else {
            Err(crate::OsoError::FromPolar)
        }
    }
}
