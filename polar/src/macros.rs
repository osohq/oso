// Uncomment these to see macro traces
// The build will fail on stable, but traces will still be printed
// #![feature(trace_macros)]
// trace_macros!(true);

/// Helper macros to create AST types
///
use std::collections::BTreeMap;
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
    ($($expr:tt)*) => {
        $crate::macros::TestHelper::<Term>::from(value!($($expr)*)).0
    };
}

impl From<(Symbol, Term)> for TestHelper<Parameter> {
    fn from(arg: (Symbol, Term)) -> Self {
        Self(Parameter {
            name: Some(arg.0),
            specializer: Some(arg.1),
        })
    }
}

impl From<Value> for TestHelper<Parameter> {
    /// Convert a Value to a parameter.  If the value is a symbol,
    /// it is used as the parameter name. Otherwise it is assumed to be
    /// a specializer.
    fn from(name: Value) -> Self {
        if let Value::Symbol(symbol) = name {
            Self(Parameter {
                name: Some(symbol),
                specializer: None,
            })
        } else {
            Self(Parameter {
                name: None,
                specializer: Some(Term::new(name)),
            })
        }
    }
}

#[macro_export]
macro_rules! param {
    ($($tt:tt)*) => {
        $crate::macros::TestHelper::<Parameter>::from($($tt)*).0
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
    ($name:expr, [$($args:expr),+]) => {
        Predicate {
            name: sym!($name),
            args: vec![
                $(term!($args)),*
            ]
        }
    }
}

#[macro_export]
macro_rules! op {
    ($op_type:ident, $($args:expr),+) => {
        Operation {
            operator: Operator::$op_type,
            args: vec![$($args),+]
        }
    };
    ($op_type:ident) => {
        Operation {
            operator: Operator::$op_type,
            args: vec![]
        }
    };
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
impl From<BTreeMap<Symbol, Term>> for TestHelper<Value> {
    fn from(other: BTreeMap<Symbol, Term>) -> Self {
        Self(Value::Dictionary(Dictionary { fields: other }))
    }
}

#[macro_export]
macro_rules! value {
    ([$($args:expr),+]) => {
        $crate::types::Value::List(vec![
            $(term!(value!($args))),*
        ])
    };
    ($arg:expr) => {
        $crate::macros::TestHelper::<Value>::from($arg).0
    };
    ("true") => {
        $crate::types::Value::Boolean(true)
    };
    ("false") => {
        $crate::types::Value::Boolean(false)
    };
}

/// Builds a list of arguments in reverse order
/// Arguments of the form `foo; bar` get built into foo specialized on bar
/// Otherwise, the argument is built depending on the type (symbols become names,
/// terms become specializers).
#[macro_export]
macro_rules! args {
    () => {
        vec![]
    };
    // this is gross: maybe match a <comma plus trailing tokens>
    ($name:expr $(, $($tt:tt)*)?) => {{
        let mut v = args!($($($tt)*)?);
        v.push(param!(value!($name)));
        v
    }};
    ($name:expr ; $spec:expr $(, $($tt:tt)*)?) => {{
        let mut v = args!($($($tt)*)?);
        v.push(param!((sym!($name), term!($spec))));
        v
    }};
}

#[macro_export]
macro_rules! rule {
    ($name:expr, [$($args:tt)*] => $($body:expr),+) => {{
        let mut params = args!($($args)*);
        params.reverse();
        Rule {
            name: sym!($name),
            params,
            body: term!(op!(And, $(term!($body)),+)),
        }}
    };
    ($name:expr, [$($args:tt)*]) => {{
        let mut params = args!($($args)*);
        params.reverse();
        Rule {
            name: sym!($name),
            params,
            body: term!(op!(And)),
        }
    }};
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
