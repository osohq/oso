/// Helper macros to create AST types
///
use std::sync::atomic::{AtomicU64, Ordering};

use crate::types::*;

pub const ORD: Ordering = Ordering::SeqCst;
pub static NEXT_ID: AtomicU64 = AtomicU64::new(0);

/// Special struct which is way more eager at implementing `From`
/// for a bunch of things, so that in the macros we can use `TestHelper<Term>::from`
/// and try and convert things as often as possible.
pub struct TestHelper<T>(pub T);

impl<T> From<T> for TestHelper<T> {
    fn from(other: T) -> Self {
        Self(other)
    }
}

impl From<Value> for TestHelper<Term> {
    fn from(other: Value) -> Self {
        Self(Term {
            id: 0, //NEXT_ID.fetch_add(1, ORD),
            offset: 0,
            value: other,
        })
    }
}

#[macro_export]
macro_rules! term {
    ($arg:expr) => {
        $crate::macros::TestHelper::<Term>::from($arg).0
    };
}

impl<S: AsRef<str>> From<S> for TestHelper<InstanceLiteral> {
    fn from(other: S) -> Self {
        Self(InstanceLiteral {
            tag: Symbol(other.as_ref().to_string()),
            fields: Dictionary::new(),
        })
    }
}

#[macro_export]
macro_rules! instance {
    ($instance:expr) => {
        $crate::macros::TestHelper::<Instance>::from($arg).0
    };
}

impl<S: AsRef<str>> From<S> for TestHelper<Symbol> {
    fn from(other: S) -> Self {
        Self(Symbol(other.as_ref().to_string()))
    }
}

#[macro_export]
macro_rules! sym {
    ($arg:expr) => {
        $crate::macros::TestHelper::<Symbol>::from($arg).0
    };
}

#[macro_export]
macro_rules! pred {
    (".", $($args:expr),+) => {
        Operation {
            operator: Operator::Dot,
            args: vec![
                $(term!(value!($args))),*
            ]
        }
    };
    ($name:expr, $($args:expr),+) => {
        Predicate {
            name: sym!($name),
            args: vec![
                $(term!(value!($args))),*
            ]
        }
    }
}

impl From<i64> for TestHelper<Value> {
    fn from(other: i64) -> Self {
        Self(Value::Integer(other))
    }
}

impl From<&str> for TestHelper<Value> {
    fn from(other: &str) -> Self {
        Self(Value::String(other.to_string()))
    }
}

impl From<bool> for TestHelper<Value> {
    fn from(other: bool) -> Self {
        Self(Value::Boolean(other))
    }
}

impl From<InstanceLiteral> for TestHelper<Value> {
    fn from(other: InstanceLiteral) -> Self {
        Self(Value::InstanceLiteral(other))
    }
}
impl From<Predicate> for TestHelper<Value> {
    fn from(other: Predicate) -> Self {
        Self(Value::Call(other))
    }
}
impl From<Operation> for TestHelper<Value> {
    fn from(other: Operation) -> Self {
        Self(Value::Expression(other))
    }
}
impl From<TermList> for TestHelper<Value> {
    fn from(other: TermList) -> Self {
        Self(Value::List(other))
    }
}
impl From<Symbol> for TestHelper<Value> {
    fn from(other: Symbol) -> Self {
        Self(Value::Symbol(other))
    }
}

#[macro_export]
macro_rules! value {
    ($arg:expr) => {
        $crate::macros::TestHelper::<Value>::from($arg).0
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
    (@and $($args:expr),*) => {
        $crate::types::Value::Expression($crate::types::Operation {
            operator: $crate::types::Operator::And,
            args: {
                vec![
                    $(term!(value!($args))),*
                ]
            }
        })
    };
}

#[macro_export]
macro_rules! rule {
    ($name:expr, $($args:expr),+ => $($body:expr),*) => {
        Rule {
            name: sym!($name),
            params: vec![
                $(term!(value!($args))),*
            ],
            body: term!(value!(@and $($body),*)),
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
