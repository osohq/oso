// Uncomment these to see macro traces
// The build will fail on stable, but traces will still be printed
// #![feature(trace_macros)]
// trace_macros!(true);

/// Helper macros to create AST types
///
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::partial::Constraints;
use crate::rules::*;
use crate::terms::*;

pub const ORD: Ordering = Ordering::SeqCst;
pub static NEXT_ID: AtomicU64 = AtomicU64::new(0);

#[macro_export]
macro_rules! value {
    ([$($args:expr),*]) => {
        $crate::terms::Value::List(vec![
            $(term!(value!($args))),*
        ])
    };
    ($arg:expr) => {
        $crate::macros::TestHelper::<Value>::from($arg).0
    };
}

#[macro_export]
macro_rules! values {
    ($([$($args:expr),*]),*) => {
        vec![$(values!($($args),*)),*]
    };
    ($($args:expr),*) => {
        vec![$(value!($args)),*]
    };
}

#[macro_export]
macro_rules! term {
    ($($expr:tt)*) => {
        $crate::macros::TestHelper::<Term>::from(value!($($expr)*)).0
    };
}

#[macro_export]
macro_rules! pattern {
    ($arg:expr) => {
        $crate::macros::TestHelper::<Pattern>::from($arg).0
    };
}

#[macro_export]
macro_rules! param {
    ($($tt:tt)*) => {
        $crate::macros::TestHelper::<Parameter>::from($($tt)*).0
    };
}

#[macro_export]
macro_rules! instance {
    ($instance:expr) => {
        InstanceLiteral {
            tag: sym!($instance),
            fields: Dictionary::new(),
        }
    };
    ($tag:expr, $fields:expr) => {
        InstanceLiteral {
            tag: sym!($tag),
            fields: $crate::macros::TestHelper::<Dictionary>::from($fields).0,
        }
    };
}

#[macro_export]
macro_rules! partial {
    ($arg:expr) => {
        Value::Partial(Constraints::new(sym!($arg)))
    };
    ($arg:expr, [$($args:expr),*]) => {
        {
            let mut constraint = Constraints::new(sym!($arg));
            $(
                constraint.add_constraint($args);
            )*
            constraint
        }
    };
}

#[macro_export]
macro_rules! sym {
    ($arg:expr) => {
        $crate::macros::TestHelper::<Symbol>::from($arg).0
    };
}

#[macro_export]
macro_rules! string {
    ($arg:expr) => {
        Value::String($arg.into())
    };
}

// TODO: support kwargs
#[macro_export]
macro_rules! call {
    ($name:expr) => {
        Call {
            name: sym!($name),
            args: vec![],
            kwargs: None
        }
    };
    ($name:expr, [$($args:expr),*]) => {
        Call {
            name: sym!($name),
            args: vec![
                $(term!($args)),*
            ],
            kwargs: None
        }
    };
    ($name:expr, [$($args:expr),*], $fields:expr) => {
        Call {
            name: sym!($name),
            args: vec![
                $(term!($args)),*
            ],
            kwargs: Some($fields)
        }
    };
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

#[macro_export]
macro_rules! dict {
    ($arg:expr) => {
        $crate::macros::TestHelper::<Dictionary>::from($arg).0
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
        Self(Term::new_from_test(other))
    }
}

// TODO change this
// TODO(gj): TODONE?
impl From<(Symbol, Term)> for TestHelper<Parameter> {
    fn from(arg: (Symbol, Term)) -> Self {
        let specializer = match arg.1.value().clone() {
            Value::Dictionary(dict) => value!(pattern!(dict)),
            Value::InstanceLiteral(lit) => value!(pattern!(lit)),
            v => v,
        };
        Self(Parameter {
            parameter: arg.1.clone_with_value(Value::Variable(arg.0)),
            specializer: Some(term!(specializer)),
        })
    }
}

impl From<Value> for TestHelper<Parameter> {
    /// Convert a Value to a parameter.  If the value is a symbol,
    /// it is used as the parameter name. Otherwise it is assumed to be
    /// a specializer.
    fn from(name: Value) -> Self {
        Self(Parameter {
            parameter: Term::new_from_test(name),
            specializer: None,
        })
    }
}

impl<S: AsRef<str>> From<S> for TestHelper<Symbol> {
    fn from(other: S) -> Self {
        Self(Symbol(other.as_ref().to_string()))
    }
}

impl From<BTreeMap<Symbol, Term>> for TestHelper<Dictionary> {
    fn from(other: BTreeMap<Symbol, Term>) -> Self {
        Self(Dictionary { fields: other })
    }
}

impl From<i64> for TestHelper<Value> {
    fn from(other: i64) -> Self {
        Self(Value::Number(other.into()))
    }
}

impl From<f64> for TestHelper<Value> {
    fn from(other: f64) -> Self {
        Self(Value::Number(other.into()))
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
impl From<Call> for TestHelper<Value> {
    fn from(other: Call) -> Self {
        Self(Value::Call(other))
    }
}
impl From<Pattern> for TestHelper<Value> {
    fn from(other: Pattern) -> Self {
        Self(Value::Pattern(other))
    }
}
impl From<Operation> for TestHelper<Value> {
    fn from(other: Operation) -> Self {
        Self(Value::Expression(other))
    }
}
impl From<Constraints> for TestHelper<Value> {
    fn from(other: Constraints) -> Self {
        Self(Value::Partial(other))
    }
}
impl From<TermList> for TestHelper<Value> {
    fn from(other: TermList) -> Self {
        Self(Value::List(other))
    }
}
impl From<Symbol> for TestHelper<Value> {
    fn from(other: Symbol) -> Self {
        Self(Value::Variable(other))
    }
}
impl From<BTreeMap<Symbol, Term>> for TestHelper<Value> {
    fn from(other: BTreeMap<Symbol, Term>) -> Self {
        Self(Value::Dictionary(Dictionary { fields: other }))
    }
}

impl From<Dictionary> for TestHelper<Pattern> {
    fn from(other: Dictionary) -> Self {
        Self(Pattern::Dictionary(other))
    }
}
impl From<BTreeMap<Symbol, Term>> for TestHelper<Pattern> {
    fn from(other: BTreeMap<Symbol, Term>) -> Self {
        Self(Pattern::Dictionary(dict!(other)))
    }
}
impl From<InstanceLiteral> for TestHelper<Pattern> {
    fn from(other: InstanceLiteral) -> Self {
        Self(Pattern::Instance(other))
    }
}
impl From<Pattern> for TestHelper<Term> {
    fn from(other: Pattern) -> Self {
        Self(Term::new_from_test(value!(other)))
    }
}
