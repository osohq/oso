//! # Formatting
//!
//! There are three forms of formatting within Polar:
//!
//! 1. Debug strings: super verbose, mostly Rust-auto derived from fmt::Debug trait
//! 2. Display string: nice user-facing versions, which could be used for things like a debugger
//! 3. Polar strings: not always implemented, but is same syntax the parser accepts
//!

pub use display::*;

pub use to_polar::*;

pub mod display {
    use std::fmt;

    use super::ToPolarString;
    use crate::vm::Goal;

    impl fmt::Display for Goal {
        fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Goal::Lookup { dict, field, value } => write!(
                    fmt,
                    "Lookup({}.{} = {})",
                    dict.to_polar(),
                    field.to_polar(),
                    value.to_polar()
                ),
                Goal::LookupExternal {
                    instance_id,
                    field,
                    value,
                    ..
                } => write!(
                    fmt,
                    "LookupExternal({}.{} = {})",
                    instance_id,
                    field.to_polar(),
                    value.to_polar(),
                ),
                Goal::Query { term } => write!(fmt, "Query({})", term.to_polar()),
                Goal::Unify { left, right } => {
                    write!(fmt, "Unify({}, {})", left.to_polar(), right.to_polar())
                }
                g => write!(fmt, "{:?}", g),
            }
        }
    }
}

pub mod to_polar {
    use crate::types::*;

    /// Helper method: uses the operator precedence to determine if `t`
    /// has a lower precedence than `op`.
    fn has_lower_pred(op: Operator, t: &Term) -> bool {
        match t.value {
            Value::Expression(Operation {
                operator: other, ..
            }) => op.precedence() > other.precedence(),
            _ => false,
        }
    }

    fn to_polar_parens(op: Operator, t: &Term) -> String {
        if has_lower_pred(op, t) {
            format!("({})", t.to_polar())
        } else {
            t.to_polar()
        }
    }

    /// Effectively works as a reverse-parser. Allows types to be turned
    /// back into polar-parseable strings.
    pub trait ToPolarString {
        fn to_polar(&self) -> String;
    }

    impl ToPolarString for Dictionary {
        fn to_polar(&self) -> String {
            let fields = self
                .fields
                .iter()
                .map(|(k, v)| format!("{}: {}", k.to_polar(), v.to_polar()))
                .collect::<Vec<String>>()
                .join(", ");
            format!("{{{}}}", fields)
        }
    }

    impl ToPolarString for ExternalInstance {
        fn to_polar(&self) -> String {
            format!("^{{id: {}}}", self.instance_id)
        }
    }

    impl ToPolarString for InstanceLiteral {
        fn to_polar(&self) -> String {
            format!("{}{}", self.tag.to_polar(), self.fields.to_polar())
        }
    }

    impl ToPolarString for Operator {
        fn to_polar(&self) -> String {
            use Operator::*;
            match self {
                Not => "!",
                Mul => "*",
                Div => "/",
                Add => "+",
                Sub => "-",
                Eq => "==",
                Geq => ">=",
                Leq => "<=",
                Neq => "!=",
                Gt => ">",
                Lt => "<",
                Or => "|",
                And => ",",
                Make => "make",
                Dot => ".",
                Unify => "=",
            }
            .to_string()
        }
    }

    impl ToPolarString for Operation {
        fn to_polar(&self) -> String {
            use Operator::*;
            // Adds parentheses when sub expressions have lower precedence (which is what you would have had to have during initial parse)
            // Lets us spit out strings that would reparse to the same ast.
            match self.operator {
                // `Make` formats as a predicate
                Make => format!(
                    "make({})",
                    self.args
                        .iter()
                        .map(|t| to_polar_parens(self.operator, t))
                        .collect::<Vec<String>>()
                        .join(",")
                ),
                // `Dot` sometimes formats as a predicate
                Dot => {
                    if self.args.len() == 2 {
                        format!("{}.{}", self.args[0].to_polar(), self.args[1].to_polar())
                    } else {
                        format!(
                            ".({})",
                            self.args
                                .iter()
                                .map(|t| to_polar_parens(self.operator, t))
                                .collect::<Vec<String>>()
                                .join(","),
                        )
                    }
                }
                // Unary operators
                Not => format!(
                    "{}{}",
                    self.operator.to_polar(),
                    to_polar_parens(self.operator, &self.args[0])
                ),
                // Binary operators
                Mul | Div | Add | Sub | Eq | Geq | Leq | Neq | Gt | Lt | Unify => format!(
                    "{}{}{}",
                    to_polar_parens(self.operator, &self.args[0]),
                    self.operator.to_polar(),
                    to_polar_parens(self.operator, &self.args[1])
                ),
                // n-ary operators
                Or | And => self
                    .args
                    .iter()
                    .map(|t| to_polar_parens(self.operator, t))
                    .collect::<Vec<String>>()
                    .join(&self.operator.to_polar()),
            }
        }
    }

    impl ToPolarString for Predicate {
        fn to_polar(&self) -> String {
            if self.args.is_empty() {
                self.name.to_polar()
            } else {
                format!(
                    "{}({})",
                    self.name.to_polar(),
                    self.args
                        .iter()
                        .map(|t| t.to_polar())
                        .collect::<Vec<String>>()
                        .join(",")
                )
            }
        }
    }

    impl ToPolarString for Rule {
        fn to_polar(&self) -> String {
            match &self.body {
                Term {
                    value:
                        Value::Expression(Operation {
                            operator: Operator::And,
                            args,
                        }),
                    ..
                } => {
                    if args.is_empty() {
                        format!(
                            "{}({});",
                            self.name.to_polar(),
                            self.params
                                .iter()
                                .map(|t| t.to_polar())
                                .collect::<Vec<String>>()
                                .join(","),
                        )
                    } else {
                        format!(
                            "{}({}) := {};",
                            self.name.to_polar(),
                            self.params
                                .iter()
                                .map(|t| t.to_polar())
                                .collect::<Vec<String>>()
                                .join(","),
                            args.iter()
                                .map(|t| t.to_polar())
                                .collect::<Vec<String>>()
                                .join(","),
                        )
                    }
                }
                _ => panic!("Not any sorta rule I parsed"),
            }
        }
    }

    impl ToPolarString for Symbol {
        fn to_polar(&self) -> String {
            self.0.to_string()
        }
    }

    impl ToPolarString for Term {
        fn to_polar(&self) -> String {
            self.value.to_polar()
        }
    }

    impl ToPolarString for Value {
        fn to_polar(&self) -> String {
            match self {
                Value::Integer(i) => format!("{}", i),
                Value::String(s) => format!("\"{}\"", s),
                Value::Boolean(b) => {
                    if *b {
                        "true".to_string()
                    } else {
                        "false".to_string()
                    }
                }
                Value::InstanceLiteral(i) => i.to_polar(),
                Value::ExternalInstanceLiteral(i) => format!("^{}", i.to_polar()),
                Value::Dictionary(i) => i.to_polar(),
                Value::ExternalInstance(i) => i.to_polar(),
                Value::Call(c) => c.to_polar(),
                Value::List(l) => format!(
                    "[{}]",
                    l.iter()
                        .map(|t| t.to_polar())
                        .collect::<Vec<String>>()
                        .join(",")
                ),
                Value::Symbol(s) => s.to_polar(),
                Value::Expression(e) => e.to_polar(),
            }
        }
    }
}
