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
    use crate::vm::*;

    impl fmt::Display for Binding {
        fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(fmt, "{} = {}", self.0.to_polar(), self.1.to_polar())
        }
    }

    impl fmt::Display for Choice {
        fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                fmt,
                "[{}] ++ [{}]",
                self.goals
                    .iter()
                    .map(|g| g.to_string())
                    .collect::<Vec<String>>()
                    .join(", "),
                self.alternatives
                    .iter()
                    .map(|alt| format!(
                        "[{}]",
                        alt.iter()
                            .map(|g| g.to_string())
                            .collect::<Vec<String>>()
                            .join(",")
                    ))
                    .collect::<Vec<String>>()
                    .join(", ")
            )
        }
    }

    impl fmt::Display for Goal {
        fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Goal::Isa { left, right } => {
                    write!(fmt, "Isa({}, {})", left.to_polar(), right.to_polar())
                }
                Goal::IsMoreSpecific { left, right, args } => write!(
                    fmt,
                    "IsMoreSpecific({} {} ({}))",
                    left.to_polar(),
                    right.to_polar(),
                    args.iter()
                        .map(|a| a.to_polar())
                        .collect::<Vec<String>>()
                        .join(", ")
                ),
                Goal::IsSubspecializer {
                    answer: _,
                    left,
                    right,
                    arg,
                } => write!(
                    fmt,
                    "IsSubspecializer({}, {}, {})",
                    left.to_polar(),
                    right.to_polar(),
                    arg.to_polar()
                ),
                Goal::Lookup { dict, field, value } => write!(
                    fmt,
                    "Lookup({}.{} = {})",
                    dict.to_polar(),
                    field.to_polar(),
                    value.to_polar()
                ),
                Goal::LookupExternal {
                    instance_id, field, ..
                } => write!(fmt, "LookupExternal({}.{})", instance_id, field.to_polar(),),
                Goal::Query { term } => write!(fmt, "Query({})", term.to_polar()),
                Goal::SortRules {
                    rules,
                    args: _,
                    outer,
                    inner,
                } => write!(
                    fmt,
                    "SortRules([{}], outer={}, inner={})",
                    rules
                        .iter()
                        .map(|rule| rule.to_polar())
                        .collect::<Vec<String>>()
                        .join(" "),
                    outer,
                    inner,
                ),
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

    /// Formats a vector of terms as a string-separated list
    /// When providing an operator, parentheses are applied suitably
    /// (see: to_polar_parens)
    fn format_args(op: Operator, args: &[Term], sep: &str) -> String {
        args.iter()
            .map(|t| to_polar_parens(op, t))
            .collect::<Vec<String>>()
            .join(sep)
    }

    /// Formats a vector of parameters
    fn format_params(args: &[Parameter], sep: &str) -> String {
        args.iter()
            .map(|parameter| parameter.to_polar())
            .collect::<Vec<String>>()
            .join(sep)
    }

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
                Make => format!("make({})", format_args(self.operator, &self.args, ",")),
                // `Dot` sometimes formats as a predicate
                Dot => {
                    if self.args.len() == 2 {
                        format!("{}.{}", self.args[0].to_polar(), self.args[1].to_polar())
                    } else {
                        format!(".({})", format_args(self.operator, &self.args, ","))
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
                Or | And => format_args(self.operator, &self.args, &self.operator.to_polar()),
            }
        }
    }

    impl ToPolarString for Parameter {
        fn to_polar(&self) -> String {
            match (&self.name, &self.specializer) {
                (Some(name), Some(specializer)) => {
                    format!("{}: {}", name.to_polar(), specializer.to_polar())
                }
                (None, Some(specializer)) => specializer.to_polar(),
                (Some(name), None) => name.to_polar(),
                (None, None) => panic!("Invalid specializer"),
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
                    format_args(Operator::And, &self.args, ",")
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
                            format_params(&self.params, ",")
                        )
                    } else {
                        format!(
                            "{}({}) := {};",
                            self.name.to_polar(),
                            format_params(&self.params, ","),
                            format_args(Operator::And, &args, ","),
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
                Value::Dictionary(i) => i.to_polar(),
                Value::ExternalInstance(i) => i.to_polar(),
                Value::Call(c) => c.to_polar(),
                Value::List(l) => format!("[{}]", format_args(Operator::And, l, ","),),
                Value::Symbol(s) => s.to_polar(),
                Value::Expression(e) => e.to_polar(),
            }
        }
    }
}
