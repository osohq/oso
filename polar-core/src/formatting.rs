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

use std::fmt::Write;

use super::{lexer::loc_to_pos, rules::*, sources::*, terms::*, traces::*};

impl Trace {
    /// Return the string representation of this `Trace`
    pub(crate) fn draw(&self, vm: &crate::vm::PolarVirtualMachine) -> String {
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
                Node::Rule(ref r) => r.to_string(),
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

/// Traverse a [`Source`](../types/struct.Source.html) line by line until `offset` is reached and
/// return the source line containing the `offset` character as well as `context_lines` lines above
/// and below it.
// @TODO: Can we have the caret under the whole range of the expression instead of just the beginning.
pub(crate) fn source_lines(source: &Source, offset: usize, context_lines: usize) -> String {
    let (target_line, target_column) = loc_to_pos(&source.src, offset);
    // Skip everything up to the first line of requested context (`target_line - context_lines`),
    // but don't overflow if `context_lines > target_line`.
    let skipped_lines = target_line.saturating_sub(context_lines);
    let mut lines = source.src.lines().enumerate().skip(skipped_lines);

    // Update `target_line` to account for skipped lines.
    let target_line = std::cmp::min(context_lines, target_line);

    // Take everything up to `target_line` as leading context.
    let prefix = lines.clone().take(target_line);

    // Take target line.
    let target = lines.nth(target_line);

    // Take _up to_ `context_lines` lines of trailing context.
    let suffix = lines.take(context_lines);

    // Combine prefix + target + suffix.
    let lines = prefix.chain(target).chain(suffix);

    // Format each line with its line number.
    let format_line = |(i, line): (usize, &str)| format!("{:03}: {}", i + 1, line);
    let mut lines: Vec<_> = lines.map(format_line).collect();

    // Insert 'indicator' line pointing at `target_column`.
    if let Some(target) = lines.get_mut(target_line) {
        // Calculate length of line number prefix.
        let prefix_len = "123: ".len();
        write!(*target, "\n{}^", " ".repeat(prefix_len + target_column)).unwrap();
    }

    lines.join("\n")
}

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
        .map(Parameter::to_string)
        .collect::<Vec<_>>()
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
        Operator::And => 2,
        Operator::Or => 1,
    }
}

/// Helper method: uses the operator precedence to determine if `t`
/// has a lower precedence than `op`.
fn has_lower_pred(op: Operator, t: &Term) -> bool {
    match t.value() {
        Value::Expression(Operation {
            operator: other, ..
        }) => precedence(&op) > precedence(other),
        _ => false,
    }
}

fn to_polar_parens(op: Operator, t: &Term) -> String {
    if has_lower_pred(op, t) {
        format!("({})", t)
    } else {
        t.to_string()
    }
}

mod display {
    use std::fmt;
    use std::sync::Arc;

    use super::to_polar::ToPolarString;
    use crate::bindings::Binding;
    use crate::numerics::Numeric;
    use crate::resource_block::Declaration;
    use crate::rules::{Parameter, Rule};
    use crate::terms::{Call, Dictionary, InstanceLiteral, Operation, Operator, Symbol, Term};
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

    impl fmt::Display for Call {
        fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(fmt, "{}", self.to_polar())
        }
    }

    impl fmt::Display for Operation {
        fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(fmt, "{}", self.to_polar())
        }
    }

    impl fmt::Display for Operator {
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
            write!(fmt, "{}", self.to_polar())
        }
    }

    impl fmt::Display for Parameter {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            write!(fmt, "{}", self.to_polar())
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

    impl fmt::Display for Declaration {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Self::Role => write!(f, "role"),
                Self::Permission => write!(f, "permission"),
                Self::Relation(_) => write!(f, "relation"),
            }
        }
    }

    impl fmt::Display for LogLevel {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Self::Trace => write!(f, "trace"),
                Self::Debug => write!(f, "debug"),
                Self::Info => write!(f, "info"),
            }
        }
    }

    impl fmt::Display for InstanceLiteral {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            write!(fmt, "{}", self.to_polar())
        }
    }

    impl fmt::Display for Dictionary {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            write!(fmt, "{}", self.to_polar())
        }
    }
}

mod to_polar {
    use std::fmt::Write;

    use crate::formatting::{format_args, format_params, to_polar_parens};
    use crate::resource_block::{BlockType, ResourceBlock, ShorthandRule};
    use crate::rules::*;
    use crate::terms::*;

    /// Effectively works as a reverse-parser. Allows types to be turned
    /// back into polar-parseable strings.
    pub(super) trait ToPolarString {
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
                format!(
                    "{} TYPE `{}`",
                    repr.clone(),
                    self.class_repr.as_ref().unwrap_or(&"UNKNOWN".to_string())
                )
            } else {
                // Print out external instances like ^{id: 123}
                // NOTE: this format is used by host libraries to enrich output
                // messages with native representations of the instances.
                format!(
                    "^{{id: {}}} TYPE `{}`",
                    self.instance_id,
                    self.class_repr.as_ref().unwrap_or(&"UNKNOWN".to_string())
                )
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

    impl Rule {
        pub(crate) fn head_as_string(&self) -> String {
            format!("{}({})", self.name, format_params(&self.params, ", "))
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
                        format!("{};", self.head_as_string())
                    } else {
                        format!(
                            "{} if {};",
                            self.head_as_string(),
                            format_args(Operator::And, args, " and "),
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

    impl ToPolarString for ShorthandRule {
        fn to_polar(&self) -> String {
            let Self {
                head,
                body: (implier, relation),
            } = self;
            if let Some((keyword, relation)) = relation {
                format!(
                    "{} if {} {} {};",
                    head.to_polar(),
                    implier.to_polar(),
                    keyword.to_polar(),
                    relation.to_polar()
                )
            } else {
                format!("{} if {};", head.to_polar(), implier.to_polar())
            }
        }
    }

    impl ToPolarString for BlockType {
        fn to_polar(&self) -> String {
            match self {
                Self::Actor => "actor".to_owned(),
                Self::Resource => "resource".to_owned(),
            }
        }
    }

    impl ToPolarString for ResourceBlock {
        fn to_polar(&self) -> String {
            let mut s = format!(
                "{} {} {{\n",
                self.block_type.to_polar(),
                self.resource.to_polar()
            );
            if let Some(ref roles) = self.roles {
                writeln!(s, "  roles = {};", roles.to_polar()).unwrap();
            }
            if let Some(ref permissions) = self.permissions {
                writeln!(s, "  permissions = {};", permissions.to_polar()).unwrap();
            }
            if let Some(ref relations) = self.relations {
                writeln!(s, "  relations = {};", relations.to_polar()).unwrap();
            }
            for rule in &self.shorthand_rules {
                writeln!(s, "  {}", rule.to_polar()).unwrap();
            }
            s += "}";
            s
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::*;

    #[test]
    fn test_source_lines() {
        let source = Source::new("hi");
        assert_eq!(source_lines(&source, 0, 0), "001: hi\n     ^");
        assert_eq!(source_lines(&source, 1, 0), "001: hi\n      ^");
        assert_eq!(source_lines(&source, 2, 0), "001: hi\n       ^");

        let src = " one\n  two\n   three\n    four\n     five\n      six\n       seven\n        eight\n         nine\n";
        let source = Source::new(src);
        let lines = source_lines(&source, 34, 2);
        let expected = indoc! {"
            003:    three
            004:     four
            005:      five
                      ^
            006:       six
            007:        seven"};
        assert_eq!(lines, expected, "\n{}", lines);
        let lines = source_lines(&source, 1, 2);
        let expected = indoc! {"
            001:  one
                  ^
            002:   two
            003:    three"};
        assert_eq!(lines, expected, "\n{}", lines);

        let source = Source::new("one\ntwo\nthree\n");
        let lines = source_lines(&source, 0, 0);
        let expected = indoc! {"
            001: one
                 ^"};
        assert_eq!(lines, expected, "\n{}", lines);
        let lines = source_lines(&source, 3, 0);
        let expected = indoc! {"
            001: one
                    ^"};
        assert_eq!(lines, expected, "\n{}", lines);
        let lines = source_lines(&source, 4, 0);
        let expected = indoc! {"
            002: two
                 ^"};
        assert_eq!(lines, expected, "\n{}", lines);
        let lines = source_lines(&source, 5, 0);
        let expected = indoc! {"
            002: two
                  ^"};
        assert_eq!(lines, expected, "\n{}", lines);
    }
}
