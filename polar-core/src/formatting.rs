//! # Formatting
//!
//! There are three main forms of formatting within Polar:
//!
//! 1. Debug strings: super verbose, mostly Rust-auto derived from fmt::Debug trait
//! 2. Display string: nice user-facing versions, which could be used for things like a debugger
//! 3. Polar strings: not always implemented, but is same syntax the parser accepts
//!
//! In addition, there are special cases like traces and sources that have their own
//! formatting requirements.

use crate::rules::*;
use crate::sources::*;
use crate::terms::*;
use crate::traces::*;
pub use display::*;
pub use to_polar::*;

impl Trace {
    /// Return the string representation of this `Trace`
    pub fn draw(&self, vm: &crate::vm::PolarVirtualMachine) -> String {
        let mut res = String::new();
        self.draw_trace(vm, 0, &mut res);
        res
    }

    fn draw_trace(&self, vm: &crate::vm::PolarVirtualMachine, nest: usize, res: &mut String) {
        if matches!(&self.node, Node::Term(term)
            if matches!(term.value(), Value::Expression(Operation { operator: Operator::And, ..})))
        {
            for c in &self.children {
                c.draw_trace(vm, nest + 1, res);
            }
        } else {
            let polar_str = match self.node {
                Node::Rule(ref r) => vm.rule_source(r),
                Node::Term(ref t) => vm.term_source(t, false),
            };
            let indented = polar_str
                .split('\n')
                .map(|s| "  ".repeat(nest) + s)
                .collect::<Vec<String>>()
                .join("\n");
            res.push_str(&indented);
            res.push_str(" [");
            if !self.children.is_empty() {
                res.push('\n');
                for c in &self.children {
                    c.draw_trace(vm, nest + 1, res);
                }
                for _ in 0..nest {
                    res.push_str("  ");
                }
            }
            res.push_str("]\n");
        }
    }
}

/// Traverse a [`Source`](../types/struct.Source.html) line by line until `offset` is reached,
/// and return the source line containing the `offset` character as well as `num_lines` lines
/// above and below it.
// @TODO: Can we have the caret under the whole range of the expression instead of just the beginning.
pub fn source_lines(source: &Source, offset: usize, num_lines: usize) -> String {
    // Sliding window of lines: current line + indicator + additional context above + below.
    let max_lines = num_lines * 2 + 2;
    let push_line = |lines: &mut Vec<String>, line: String| {
        if lines.len() == max_lines {
            lines.remove(0);
        }
        lines.push(line);
    };
    let mut index = 0;
    let mut lines = Vec::new();
    let mut target = None;
    let prefix_len = "123: ".len();
    for (lineno, line) in source.src.lines().enumerate() {
        push_line(&mut lines, format!("{:03}: {}", lineno + 1, line));
        let end = index + line.len() + 1; // Adding one to account for new line byte.
        if target.is_none() && end >= offset {
            target = Some(lineno);
            let spaces = " ".repeat(offset - index + prefix_len);
            push_line(&mut lines, format!("{}^", spaces));
        }
        index = end;
        if target.is_some() && lineno == target.unwrap() + num_lines {
            break;
        }
    }
    lines.join("\n")
}

/// Formats a vector of terms as a string-separated list
/// When providing an operator, parentheses are applied suitably
/// (see: to_polar_parens)
pub fn format_args(op: Operator, args: &[Term], sep: &str) -> String {
    args.iter()
        .map(|t| to_polar_parens(op, t))
        .collect::<Vec<String>>()
        .join(sep)
}

/// Formats a vector of parameters
pub fn format_params(args: &[Parameter], sep: &str) -> String {
    args.iter()
        .map(|parameter| parameter.to_polar())
        .collect::<Vec<String>>()
        .join(sep)
}

/// Formats a vector of rules as a string-separated list.
#[allow(clippy::ptr_arg)]
pub fn format_rules(rules: &Rules, sep: &str) -> String {
    rules
        .iter()
        .map(|rule| rule.to_polar())
        .collect::<Vec<String>>()
        .join(sep)
}

fn precedence(o: &Operator) -> i32 {
    match o {
        Operator::Print => 11,
        Operator::Debug => 11,
        Operator::New => 10,
        Operator::Cut => 10,
        Operator::ForAll => 10,
        Operator::Dot => 9,
        Operator::In => 8,
        Operator::Isa => 8,
        Operator::Mul => 7,
        Operator::Div => 7,
        Operator::Mod => 7,
        Operator::Rem => 7,
        Operator::Add => 6,
        Operator::Sub => 6,
        Operator::Eq => 5,
        Operator::Geq => 5,
        Operator::Leq => 5,
        Operator::Neq => 5,
        Operator::Gt => 5,
        Operator::Lt => 5,
        Operator::Unify => 4,
        Operator::Assign => 4,
        Operator::Not => 3,
        Operator::Or => 2,
        Operator::And => 1,
    }
}

/// Helper method: uses the operator precedence to determine if `t`
/// has a lower precedence than `op`.
fn has_lower_pred(op: Operator, t: &Term) -> bool {
    match t.value() {
        Value::Expression(Operation {
            operator: other, ..
        }) => precedence(&op) > precedence(&other),
        _ => false,
    }
}

pub fn to_polar_parens(op: Operator, t: &Term) -> String {
    if has_lower_pred(op, t) {
        format!("({})", t.to_polar())
    } else {
        t.to_polar()
    }
}

pub mod display {
    use crate::formatting::{format_args, format_params};
    use std::fmt;
    use std::sync::Arc;

    use super::ToPolarString;
    use crate::bindings::Binding;
    use crate::numerics::Numeric;
    use crate::rules::Rule;
    use crate::terms::{Operation, Operator, Symbol, Term, Value};
    use crate::vm::*;

    impl fmt::Display for Binding {
        fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(fmt, "{} = {}", self.0.to_polar(), self.1.to_polar())
        }
    }

    impl fmt::Display for Symbol {
        fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(fmt, "{}", self.0)
        }
    }

    impl fmt::Display for Term {
        fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(fmt, "{}", self.to_polar())
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
            fn fmt_rules(rules: &[Arc<Rule>]) -> String {
                rules
                    .iter()
                    .map(|rule| rule.to_polar())
                    .collect::<Vec<String>>()
                    .join(" ")
            }

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
                    left, right, arg, ..
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
                    instance, field, ..
                } => write!(
                    fmt,
                    "LookupExternal({}.{})",
                    instance.to_polar(),
                    field.to_polar(),
                ),
                Goal::PopQuery { term } => write!(fmt, "PopQuery({})", term.to_polar()),
                Goal::Query { term } => write!(fmt, "Query({})", term.to_polar()),
                Goal::Run { .. } => write!(fmt, "Run(...)"),
                Goal::FilterRules {
                    applicable_rules,
                    unfiltered_rules,
                    ..
                } => write!(
                    fmt,
                    "FilterRules([{}], [{}])",
                    fmt_rules(applicable_rules),
                    fmt_rules(unfiltered_rules),
                ),
                Goal::SortRules {
                    rules,
                    outer,
                    inner,
                    ..
                } => write!(
                    fmt,
                    "SortRules([{}], outer={}, inner={})",
                    fmt_rules(rules),
                    outer,
                    inner,
                ),
                Goal::TraceRule { trace: _ } => write!(
                    fmt,
                    "TraceRule(...)" // FIXME: draw trace?
                ),
                Goal::Unify { left, right } => {
                    write!(fmt, "Unify({}, {})", left.to_polar(), right.to_polar())
                }
                g => write!(fmt, "{:?}", g),
            }
        }
    }

    impl fmt::Display for Rule {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            match &self.body.value() {
                Value::Expression(Operation {
                    operator: Operator::And,
                    args,
                }) => {
                    if args.is_empty() {
                        write!(
                            fmt,
                            "{}({});",
                            self.name.to_polar(),
                            format_params(&self.params, ", ")
                        )
                    } else {
                        write!(
                            fmt,
                            "{}({}) if {};",
                            self.name.to_polar(),
                            format_params(&self.params, ", "),
                            format_args(Operator::And, &args, ",\n  "),
                        )
                    }
                }
                _ => panic!("Not any sorta rule I parsed"),
            }
        }
    }

    impl fmt::Display for Numeric {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Self::Integer(i) => write!(f, "{}", i),
                Self::Float(float) => write!(f, "{}", float),
            }
        }
    }
}

pub mod to_polar {
    use crate::formatting::{format_args, format_params, to_polar_parens};
    use crate::rules::*;
    use crate::terms::*;

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
            if let Some(ref repr) = self.repr {
                repr.clone()
            } else {
                format!("^{{id: {}}}", self.instance_id)
            }
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
                Not => "not",
                Mul => "*",
                Div => "/",
                Mod => "mod",
                Rem => "rem",
                Add => "+",
                Sub => "-",
                Eq => "==",
                Geq => ">=",
                Leq => "<=",
                Neq => "!=",
                Gt => ">",
                Lt => "<",
                Or => "or",
                And => "and",
                New => "new",
                Dot => ".",
                Unify => "=",
                Assign => ":=",
                In => "in",
                Cut => "cut",
                ForAll => "forall",
                Debug => "debug",
                Print => "print",
                Isa => "matches",
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
                Debug => "debug()".to_owned(),
                Print => format!("print({})", format_args(self.operator, &self.args, ", ")),
                Cut => "cut".to_owned(),
                ForAll => format!(
                    "forall({}, {})",
                    self.args[0].to_polar(),
                    self.args[1].to_polar()
                ),
                New => {
                    if self.args.len() == 1 {
                        format!("new {}", to_polar_parens(self.operator, &self.args[0]))
                    } else {
                        format!(
                            "new ({}, {})",
                            to_polar_parens(self.operator, &self.args[0]),
                            self.args[1].to_polar()
                        )
                    }
                }
                // Lookup operator
                Dot => {
                    let call_term = if let Value::String(s) = self.args[1].value() {
                        s.to_string()
                    } else {
                        self.args[1].to_polar()
                    };
                    match self.args.len() {
                        2 => format!("{}.{}", self.args[0].to_polar(), call_term),
                        3 => format!(
                            "{}.{} = {}",
                            self.args[0].to_polar(),
                            call_term,
                            self.args[2].to_polar()
                        ),
                        // Invalid
                        _ => format!(".({})", format_args(self.operator, &self.args, ", ")),
                    }
                }
                // Unary operators
                Not => format!(
                    "{} {}",
                    self.operator.to_polar(),
                    to_polar_parens(self.operator, &self.args[0])
                ),
                // Binary operators
                Mul | Div | Mod | Rem | Add | Sub | Eq | Geq | Leq | Neq | Gt | Lt | Unify
                | Isa | In | Assign => match self.args.len() {
                    2 => format!(
                        "{} {} {}",
                        to_polar_parens(self.operator, &self.args[0]),
                        self.operator.to_polar(),
                        to_polar_parens(self.operator, &self.args[1]),
                    ),
                    3 => format!(
                        "{} {} {} = {}",
                        to_polar_parens(self.operator, &self.args[0]),
                        self.operator.to_polar(),
                        to_polar_parens(self.operator, &self.args[1]),
                        to_polar_parens(self.operator, &self.args[2]),
                    ),
                    // Invalid
                    _ => format!(
                        "{}({})",
                        self.operator.to_polar(),
                        format_args(self.operator, &self.args, ", ")
                    ),
                },
                // n-ary operators
                And if self.args.is_empty() => "(true)".to_string(),
                And => format_args(
                    self.operator,
                    &self.args,
                    &format!(" {} ", self.operator.to_polar()),
                ),
                Or if self.args.is_empty() => "(false)".to_string(),
                Or => format_args(
                    self.operator,
                    &self.args,
                    &format!(" {} ", self.operator.to_polar()),
                ),
            }
        }
    }

    impl ToPolarString for Parameter {
        fn to_polar(&self) -> String {
            match &self.specializer {
                None => self.parameter.to_polar(),
                Some(specializer) => {
                    format!("{}: {}", self.parameter.to_polar(), specializer.to_polar())
                }
            }
        }
    }

    impl ToPolarString for Call {
        fn to_polar(&self) -> String {
            let args = format_args(Operator::And, &self.args, ", ");
            let combined_args = match &self.kwargs {
                Some(dict) => {
                    let kwargs = dict
                        .iter()
                        .map(|(k, v)| format!("{}: {}", k.to_polar(), v.to_polar()))
                        .collect::<Vec<String>>()
                        .join(", ");
                    if args.is_empty() {
                        kwargs
                    } else {
                        vec![args, kwargs].join(", ")
                    }
                }
                None => args,
            };
            format!("{}({})", self.name.to_polar(), combined_args)
        }
    }

    impl ToPolarString for Rule {
        fn to_polar(&self) -> String {
            match &self.body.value() {
                Value::Expression(Operation {
                    operator: Operator::And,
                    args,
                }) => {
                    if args.is_empty() {
                        format!(
                            "{}({});",
                            self.name.to_polar(),
                            format_params(&self.params, ", ")
                        )
                    } else {
                        format!(
                            "{}({}) if {};",
                            self.name.to_polar(),
                            format_params(&self.params, ", "),
                            format_args(Operator::And, &args, " and "),
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
            self.value().to_polar()
        }
    }

    impl ToPolarString for Pattern {
        fn to_polar(&self) -> String {
            match self {
                Pattern::Dictionary(d) => d.to_polar(),
                Pattern::Instance(i) => i.to_polar(),
            }
        }
    }

    impl ToPolarString for Value {
        fn to_polar(&self) -> String {
            match self {
                Value::Number(i) => format!("{}", i),
                Value::String(s) => format!("\"{}\"", s),
                Value::Boolean(b) => {
                    if *b {
                        "true".to_string()
                    } else {
                        "false".to_string()
                    }
                }
                Value::Dictionary(i) => i.to_polar(),
                Value::Pattern(i) => i.to_polar(),
                Value::ExternalInstance(i) => i.to_polar(),
                Value::Call(c) => c.to_polar(),
                Value::List(l) => format!("[{}]", format_args(Operator::And, l, ", "),),
                Value::Variable(s) => s.to_polar(),
                Value::RestVariable(s) => format!("*{}", s.to_polar()),
                Value::Expression(e) => e.to_polar(),
            }
        }
    }
}
