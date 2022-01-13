// The build will fail on stable, but traces will still be printed
// #![feature(trace_macros)]
// trace_macros!(true);

/// Helper macros to create AST types
///
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::rules::*;
use crate::terms::*;

pub const ORD: Ordering = Ordering::SeqCst;
pub static NEXT_ID: AtomicU64 = AtomicU64::new(0);

#[macro_export]
macro_rules! match_var {
    ($v:ident) => {
        crate::terms::Value::Variable(crate::terms::Variable { name: $v, .. })
    };
}

#[macro_export]
macro_rules! value {
    ([$($args:expr),* , @rest $rv:literal]) => {
        $crate::terms::Value::List(List {
            elements: vec![
                $(term!(value!($args))),*
            ],
            rest_var: Some(Variable::new($rv.to_string()))
        })
    };
    ([$($args:expr),*]) => {
        $crate::terms::Value::List($crate::terms::List {
            elements: vec![
                $(term!(value!($args))),*
            ],
            rest_var: None
        })
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
macro_rules! param {
    ($($tt:tt)*) => {
        $crate::macros::TestHelper::<Parameter>::from($($tt)*).0
    };
}

#[macro_export]
macro_rules! instance {
    ($instance:expr) => {
        crate::terms::InstanceLiteral {
            tag: sym!($instance),
            fields: crate::terms::Dictionary::new(),
        }
    };
    ($tag:expr, $fields:expr) => {
        crate::terms::InstanceLiteral {
            tag: sym!($tag),
            fields: $crate::macros::TestHelper::<crate::terms::Dictionary>::from($fields).0,
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
macro_rules! result_var {
    ($arg:expr) => {{
        let mut var = Variable::new($arg.to_string());
        var.frame = 1;
        var
    }};
}

#[macro_export]
macro_rules! var {
    ($arg:expr) => {
        $crate::macros::TestHelper::<Term>::from(
            $crate::macros::TestHelper::<Value>::from(sym!($arg)).0,
        )
        .0
    };
}

#[macro_export]
macro_rules! string {
    ($arg:expr) => {
        Value::String($arg.into())
    };
}

#[macro_export]
macro_rules! str {
    ($arg:expr) => {
        $crate::macros::TestHelper::<Term>::from(string!($arg)).0
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
            operator: crate::terms::Operator::$op_type,
            args: vec![$($args),+]
        }
    };
    ($op_type:ident) => {
        Operation {
            operator: crate::terms::Operator::$op_type,
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
            source_info: $crate::sources::SourceInfo::Test,
            required: false,
        }}
    };
    ($name:expr, [$($args:tt)*]) => {{
        let mut params = args!($($args)*);
        params.reverse();
        Rule {
            name: sym!($name),
            params,
            body: term!(op!(And)),
            source_info: $crate::sources::SourceInfo::Test,
            required: false,
        }
    }};
    // this macro variant is used exclusively to create rule *types*
    // TODO: @patrickod break into specific-purpose rule_type! macro and RuleType struct
    ($name:expr, [$($args:tt)*], $required:expr) => {{
        let mut params = args!($($args)*);
        params.reverse();
        Rule {
            name: sym!($name),
            params,
            body: term!(op!(And)),
            source_info: $crate::sources::SourceInfo::Test,
            required: $required,
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
        Self(Term::from(other))
    }
}

// TODO change this
// TODO(gj): TODONE?
impl From<(Symbol, Term)> for TestHelper<Parameter> {
    fn from(arg: (Symbol, Term)) -> Self {
        let specializer = match arg.1.value().clone() {
            Value::Dictionary(dict) => value!(dict),
            v => v,
        };
        Self(Parameter {
            parameter: arg
                .1
                .clone_with_value(Value::Variable(Variable::new(arg.0 .0))),
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
            parameter: Term::from(name),
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
impl From<String> for TestHelper<Value> {
    fn from(other: String) -> Self {
        Self(Value::String(other))
    }
}

impl From<bool> for TestHelper<Value> {
    fn from(other: bool) -> Self {
        Self(Value::Boolean(other))
    }
}

impl From<Call> for TestHelper<Value> {
    fn from(other: Call) -> Self {
        Self(Value::Call(other))
    }
}
impl From<Dictionary> for TestHelper<Value> {
    fn from(other: Dictionary) -> Self {
        Self(Value::Dictionary(other))
    }
}
impl From<InstanceLiteral> for TestHelper<Value> {
    fn from(other: InstanceLiteral) -> Self {
        Self(Value::InstanceLiteral(other))
    }
}
impl From<Operation> for TestHelper<Value> {
    fn from(other: Operation) -> Self {
        Self(Value::Expression(other))
    }
}
impl From<List> for TestHelper<Value> {
    fn from(other: List) -> Self {
        Self(Value::List(other))
    }
}
impl From<TermList> for TestHelper<Value> {
    fn from(other: TermList) -> Self {
        Self(Value::List(List {
            elements: other,
            rest_var: None,
        }))
    }
}
impl From<Symbol> for TestHelper<Value> {
    fn from(other: Symbol) -> Self {
        Self(Value::Variable(Variable::new(other.0)))
    }
}
impl From<Variable> for TestHelper<Value> {
    fn from(other: Variable) -> Self {
        Self(Value::Variable(other))
    }
}
impl From<BTreeMap<Symbol, Term>> for TestHelper<Value> {
    fn from(other: BTreeMap<Symbol, Term>) -> Self {
        Self(Value::Dictionary(Dictionary { fields: other }))
    }
}

impl<'a, T> From<&'a T> for TestHelper<Value>
where
    T: Clone + Into<TestHelper<Value>>,
{
    fn from(other: &'a T) -> Self {
        other.clone().into()
    }
}
