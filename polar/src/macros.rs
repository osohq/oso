/// Helper macros to create AST types
///
use std::sync::atomic::{AtomicU64, Ordering};

use crate::types::*;

pub const ORD: Ordering = Ordering::SeqCst;
pub const NEXT_ID: AtomicU64 = AtomicU64::new(0);

/// Special struct which is way more eager at implementing `From`
/// for a bunch of things, so that in the macros we can use `FromHelper<Term>::from`
/// and try and convert things as often as possible.
pub struct FromHelper<T>(pub T);

impl<T> From<T> for FromHelper<T> {
    fn from(other: T) -> Self {
        Self(other)
    }
}

impl From<Value> for FromHelper<Term> {
    fn from(other: Value) -> Self {
        Self(Term {
            id: NEXT_ID.fetch_add(1, ORD),
            offset: 0xCAFE,
            value: other,
        })
    }
}

#[macro_export]
macro_rules! term {
    ($arg:expr) => {
        $crate::macros::FromHelper::<Term>::from($arg).0
    };
}

impl<S: AsRef<str>> From<S> for FromHelper<Instance> {
    fn from(other: S) -> Self {
        Self(Instance {
            class: other.as_ref().to_string(),
            external_id: NEXT_ID.fetch_add(1, ORD),
        })
    }
}

#[macro_export]
macro_rules! instance {
    ($instance:expr) => {
        $crate::macros::FromHelper::<Instance>::from($arg).0
    };
}

impl<S: AsRef<str>> From<S> for FromHelper<Symbol> {
    fn from(other: S) -> Self {
        Self(Symbol(other.as_ref().to_string()))
    }
}

#[macro_export]
macro_rules! sym {
    ($arg:expr) => {
        $crate::macros::FromHelper::<Symbol>::from($arg).0
    };
}

#[macro_export]
macro_rules! pred {
    ($name:expr, $($args:expr),+) => {
        Predicate {
            name: $name.to_string(),
            args: vec![
                $(term!(value!($args))),*
            ]
        }
    }
}

impl From<i64> for FromHelper<Value> {
    fn from(other: i64) -> Self {
        Self(Value::Integer(other))
    }
}

impl From<&str> for FromHelper<Value> {
    fn from(other: &str) -> Self {
        Self(Value::String(other.to_string()))
    }
}

impl From<bool> for FromHelper<Value> {
    fn from(other: bool) -> Self {
        Self(Value::Boolean(other))
    }
}

impl From<Instance> for FromHelper<Value> {
    fn from(other: Instance) -> Self {
        Self(Value::Instance(other))
    }
}
impl From<Predicate> for FromHelper<Value> {
    fn from(other: Predicate) -> Self {
        Self(Value::Call(other))
    }
}
impl From<TermList> for FromHelper<Value> {
    fn from(other: TermList) -> Self {
        Self(Value::List(other))
    }
}
impl From<Symbol> for FromHelper<Value> {
    fn from(other: Symbol) -> Self {
        Self(Value::Symbol(other))
    }
}

#[macro_export]
macro_rules! value {
    ($arg:expr) => {
        $crate::macros::FromHelper::<Value>::from($arg).0
    };
    (@int $arg:expr) => {
        $crate::types::Value::Integer(i64::from($arg))
    };
    (@str $arg:expr) => {
        $crate::types::Value::String($arg.to_string())
    };
    ("true") => {
        $crate::types::Value::Boolean(true)
    };
    ("false") => {
        $crate::types::Value::Boolean(false)
    };
    (@instance $arg:expr) => {
        $crate::types::Value::Instance($arg)
    };
    (@pred $arg:expr) => {
        $crate::types::Value::Predicate($arg)
    };
    (@tl $($args:expr),+) => {
        $crate::types::Value::List(vec![
            $(term!(value!($args))),*
        ])
    };
    (@sym $arg:expr) => {
        $crate::types::Value::Symbol(sym!($arg))
    };
}

#[macro_export]
macro_rules! rule {
    ($name:expr, $($args:expr),+ => $($body:expr),*) => {
        Rule {
            name: $name.to_string(),
            params: vec![
                $(term!(value!($args))),*
            ],
            body: vec![
                $(term!(value!($body))),*
            ],
        }
    }
}
// #[macro_export]
// macro_rules! list {
//     ([]) => {{
//         Term::empty_list()
//     }};
//     ([$head:expr , [ $($tail:tt)* ]]) => {{
//         let list = list!([$($tail)*]);
//         list.insert_list($head)
//     }};
// }
