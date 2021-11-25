use std::fmt;

use indoc::formatdoc;
use serde::{Deserialize, Serialize};

use super::{
    diagnostic::{Context, Range},
    formatting::to_polar::ToPolarString,
    kb::KnowledgeBase,
    rules::Rule,
    sources::Source,
    terms::{Operation, Symbol, Term},
};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(into = "FormattedPolarError")]
pub struct PolarError {
    pub kind: ErrorKind,
    pub context: Option<Context>,
}

impl PolarError {
    pub fn kind(&self) -> String {
        use ErrorKind::*;
        use OperationalError::*;
        use ParseError::*;
        use RuntimeError::*;
        use ValidationError::*;

        match self.kind {
            Parse(IntegerOverflow { .. }) => "ParseError::IntegerOverflow",
            Parse(InvalidTokenCharacter { .. }) => "ParseError::InvalidTokenCharacter",
            Parse(InvalidToken { .. }) => "ParseError::InvalidToken",
            Parse(UnrecognizedEOF { .. }) => "ParseError::UnrecognizedEOF",
            Parse(UnrecognizedToken { .. }) => "ParseError::UnrecognizedToken",
            Parse(ExtraToken { .. }) => "ParseError::ExtraToken",
            Parse(ReservedWord { .. }) => "ParseError::ReservedWord",
            Parse(InvalidFloat { .. }) => "ParseError::InvalidFloat",
            Parse(WrongValueType { .. }) => "ParseError::WrongValueType",
            Parse(DuplicateKey { .. }) => "ParseError::DuplicateKey",
            Runtime(Application { .. }) => "RuntimeError::Application",
            Runtime(ArithmeticError { .. }) => "RuntimeError::ArithmeticError",
            Runtime(FileLoading { .. }) => "RuntimeError::FileLoading",
            Runtime(IncompatibleBindings { .. }) => "RuntimeError::IncompatibleBindings",
            Runtime(QueryTimeout { .. }) => "RuntimeError::QueryTimeout",
            Runtime(StackOverflow { .. }) => "RuntimeError::StackOverflow",
            Runtime(TypeError { .. }) => "RuntimeError::TypeError",
            Runtime(UnhandledPartial { .. }) => "RuntimeError::UnhandledPartial",
            Runtime(Unsupported { .. }) => "RuntimeError::Unsupported",
            Runtime(DataFilteringFieldMissing { .. }) => "RuntimeError::DataFilteringFieldMissing",
            Runtime(DataFilteringUnsupportedOp { .. }) => {
                "RuntimeError::DataFilteringUnsupportedOp"
            }
            Runtime(InvalidRegistration { .. }) => "RuntimeError::InvalidRegistration",
            Runtime(InvalidState { .. }) => "RuntimeError::InvalidState",
            Operational(Serialization { .. }) => "OperationalError::Serialization",
            Operational(Unknown) => "OperationalError::Unknown",
            Validation(InvalidRule { .. }) => "ValidationError::InvalidRule",
            Validation(InvalidRuleType { .. }) => "ValidationError::InvalidRuleType",
            Validation(ResourceBlock { .. }) => "ValidationError::ResourceBlock",
            Validation(UndefinedRuleCall { .. }) => "ValidationError::UndefinedRuleCall",
            Validation(SingletonVariable { .. }) => "ValidationError::SingletonVariable",
            Validation(UnregisteredClass { .. }) => "ValidationError::UnregisteredClass",
            Validation(MissingRequiredRule { .. }) => "ValidationError::MissingRequiredRule",
        }
        .to_owned()
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct FormattedPolarError {
    pub kind: ErrorKind,
    pub formatted: String,
}

impl From<PolarError> for FormattedPolarError {
    fn from(other: PolarError) -> Self {
        Self {
            formatted: other.to_string(),
            kind: other.kind,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorKind {
    Parse(ParseError),
    Runtime(RuntimeError),
    Operational(OperationalError),
    Validation(ValidationError),
}

pub type PolarResult<T> = std::result::Result<T, PolarError>;

impl std::error::Error for PolarError {}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorKind::Parse(e) => write!(f, "{}", e)?,
            ErrorKind::Runtime(e) => write!(f, "{}", e)?,
            ErrorKind::Operational(e) => write!(f, "{}", e)?,
            ErrorKind::Validation(e) => write!(f, "{}", e)?,
        }
        Ok(())
    }
}

impl fmt::Display for PolarError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.kind)?;
        if let Some(ref context) = self.context {
            write!(f, "{}", context)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParseError {
    IntegerOverflow {
        token: String,
        loc: usize,
    },
    InvalidTokenCharacter {
        token: String,
        c: char,
        loc: usize,
    },
    InvalidToken {
        loc: usize,
    },
    #[allow(clippy::upper_case_acronyms)]
    UnrecognizedEOF {
        loc: usize,
    },
    UnrecognizedToken {
        token: String,
        loc: usize,
    },
    ExtraToken {
        token: String,
        loc: usize,
    },
    ReservedWord {
        token: String,
        loc: usize,
    },
    InvalidFloat {
        token: String,
        loc: usize,
    },
    WrongValueType {
        loc: usize,
        term: Term,
        expected: String,
    },
    DuplicateKey {
        loc: usize,
        key: String,
    },
}

impl ParseError {
    pub fn with_context(self, source: Source) -> PolarError {
        use ParseError::*;

        let span = match &self {
            // These errors track `loc` (left bound) and `token`, and we calculate right bound
            // as `loc + token.len()`.
            DuplicateKey { key: token, loc }
            | ExtraToken { token, loc }
            | IntegerOverflow { token, loc }
            | InvalidFloat { token, loc }
            | ReservedWord { token, loc }
            | UnrecognizedToken { token, loc } => (*loc, loc + token.len()),

            // These errors track `loc` and only pertain to a single character, so right bound
            // of span is also `loc`.
            InvalidTokenCharacter { loc, .. } | InvalidToken { loc } | UnrecognizedEOF { loc } => {
                (*loc, *loc)
            }

            // These errors track `term`, from which we calculate the span.
            WrongValueType { term, .. } => term.span().expect("always from parser"),
        };
        let range = Range::from_span(&source.src, span);

        PolarError {
            context: Some(Context { range, source }),
            kind: ErrorKind::Parse(self),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::IntegerOverflow { token, .. } => {
                write!(f, "'{}' caused an integer overflow", token.escape_debug())
            }
            Self::InvalidTokenCharacter { token, c, .. } => write!(
                f,
                "'{}' is not a valid character. Found in {}",
                c.escape_debug(),
                token.escape_debug()
            ),
            Self::InvalidToken { .. } => write!(f, "found an unexpected sequence of characters"),
            Self::UnrecognizedEOF { .. } => write!(
                f,
                "hit the end of the file unexpectedly. Did you forget a semi-colon"
            ),
            Self::UnrecognizedToken { token, .. } => write!(
                f,
                "did not expect to find the token '{}'",
                token.escape_debug()
            ),
            Self::ExtraToken { token, .. } => write!(
                f,
                "did not expect to find the token '{}'",
                token.escape_debug()
            ),
            Self::ReservedWord { token, .. } => write!(
                f,
                "{} is a reserved Polar word and cannot be used here",
                token.escape_debug()
            ),
            Self::InvalidFloat { token, .. } => write!(
                f,
                "{} was parsed as a float, but is invalid",
                token.escape_debug()
            ),
            Self::WrongValueType { term, expected, .. } => {
                write!(f, "Wrong value type: {}. Expected a {}", term, expected)
            }
            Self::DuplicateKey { key, .. } => {
                write!(f, "Duplicate key: {}", key)
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuntimeError {
    ArithmeticError {
        /// Term<Operation> where the error arose, tracked for lexical context.
        term: Term,
    },
    Unsupported {
        msg: String,
        /// Term where the error arose, tracked for lexical context.
        term: Term,
    },
    TypeError {
        msg: String,
        stack_trace: String,
        /// Term where the error arose, tracked for lexical context.
        term: Term,
    },
    StackOverflow {
        msg: String,
    },
    QueryTimeout {
        msg: String,
    },
    Application {
        msg: String,
        stack_trace: String,
        /// Option<Term> where the error arose, tracked for lexical context.
        term: Option<Term>,
    },
    // TODO(gj): consider moving to ValidationError.
    FileLoading {
        msg: String,
    },
    IncompatibleBindings {
        msg: String,
    },
    UnhandledPartial {
        var: Symbol,
        /// Term where the error arose, tracked for lexical context.
        term: Term,
    },
    DataFilteringFieldMissing {
        var_type: String,
        field: String,
    },
    DataFilteringUnsupportedOp {
        operation: Operation,
    },
    // TODO(gj): consider moving to ValidationError.
    InvalidRegistration {
        sym: Symbol,
        msg: String,
    },
    /// An invariant has been broken internally.
    InvalidState {
        msg: String,
    },
}

impl RuntimeError {
    pub fn with_context(self, kb: &KnowledgeBase) -> PolarError {
        use RuntimeError::*;

        let context = match &self {
            // These errors sometimes track `term`, from which we derive context.
            Application { term, .. } => term
                .as_ref()
                .and_then(Term::span)
                .zip(term.as_ref().and_then(|t| kb.get_term_source(t))),

            // These errors track `term`, from which we derive the context.
            ArithmeticError { term }
            | TypeError { term, .. }
            | UnhandledPartial { term, .. }
            | Unsupported { term, .. } => term.span().zip(kb.get_term_source(term)),

            // These errors never have context.
            StackOverflow { .. }
            | QueryTimeout { .. }
            | FileLoading { .. }
            | IncompatibleBindings { .. }
            | DataFilteringFieldMissing { .. }
            | DataFilteringUnsupportedOp { .. }
            | InvalidRegistration { .. }
            | InvalidState { .. } => None,
        };

        let context = context.map(|(span, source)| Context {
            range: Range::from_span(&source.src, span),
            source,
        });

        PolarError {
            kind: ErrorKind::Runtime(self),
            context,
        }
    }

    pub fn unsupported<A>(msg: String, term: Term) -> Result<A, RuntimeError> {
        Err(Self::Unsupported { msg, term })
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ArithmeticError { term } => write!(f, "Arithmetic error: {}", term),
            Self::Unsupported { msg, .. } => write!(f, "Not supported: {}", msg),
            Self::TypeError {
                msg, stack_trace, ..
            } => {
                writeln!(f, "{}", stack_trace)?;
                write!(f, "Type error: {}", msg)
            }
            Self::StackOverflow { msg } => {
                write!(f, "{}", msg)
            }
            Self::QueryTimeout { msg } => write!(f, "Query timeout: {}", msg),
            Self::Application {
                msg, stack_trace, ..
            } => {
                writeln!(f, "{}", stack_trace)?;
                write!(f, "Application error: {}", msg)
            }
            Self::FileLoading { msg } => write!(f, "Problem loading file: {}", msg),
            Self::IncompatibleBindings { msg } => {
                write!(f, "Attempted binding was incompatible: {}", msg)
            }
            Self::UnhandledPartial { var, term } => {
                write!(
                    f,
                    "Found an unhandled partial in the query result: {var}

This can happen when there is a variable used inside a rule
which is not related to any of the query inputs.

For example: f(_x) if y.a = 1 and y.b = 2;

In this example, the variable `y` is constrained by `a = 1 and b = 2`,
but we cannot resolve these constraints without further information.

The unhandled partial is for variable {var}.
The expression is: {expr}
",
                    var = var,
                    expr = term,
                )
            }
            Self::DataFilteringFieldMissing { var_type, field } => {
                let msg = formatdoc!(
                    r#"Unregistered field or relation: {var_type}.{field}

                    Please include `{field}` in the `fields` parameter of your
                    `register_class` call for {var_type}.  For example, in Python:

                        oso.register_class({var_type}, fields={{
                            "{field}": <type or relation>
                        }})

                    For more information please refer to our documentation:
                        https://docs.osohq.com/guides/data_filtering.html
                    "#,
                    var_type = var_type,
                    field = field
                );
                write!(f, "{}", msg)
            }
            Self::DataFilteringUnsupportedOp { operation } => {
                let msg = formatdoc!(
                    r#"Unsupported operation: {}

                    This operation is not supported for data filtering.
                    For more information please refer to our documentation:
                        https://docs.osohq.com/guides/data_filtering.html
                    "#,
                    operation.to_polar()
                );
                write!(f, "{}", msg)
            }
            Self::InvalidRegistration { sym, msg } => {
                write!(f, "Invalid attempt to register '{}': {}", sym, msg)
            }
            // TODO(gj): move this back to `OperationalError` during The Next Great Diagnostic
            // Refactor.
            Self::InvalidState { msg } => write!(f, "Invalid state: {}", msg),
        }
    }
}

// NOTE(gj): both of these errors are only constructed/used in the `polar-c-api` crate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationalError {
    Serialization {
        msg: String,
    },
    /// Rust panics caught in the `polar-c-api` crate.
    Unknown,
}

impl From<OperationalError> for PolarError {
    fn from(err: OperationalError) -> Self {
        Self {
            kind: ErrorKind::Operational(err),
            context: None,
        }
    }
}

impl fmt::Display for OperationalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Serialization { msg } => write!(f, "Serialization error: {}", msg),
            Self::Unknown => write!(
                f,
                "We hit an unexpected error.\n\
                Please submit a bug report at <https://github.com/osohq/oso/issues>"
            ),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationError {
    MissingRequiredRule {
        rule_type: Rule,
    },
    InvalidRule {
        /// Rule where the error arose, tracked for lexical context.
        rule: Rule,
        msg: String,
    },
    InvalidRuleType {
        /// Rule type where the error arose, tracked for lexical context.
        rule_type: Rule,
        msg: String,
    },
    UndefinedRuleCall {
        /// Term<Call> where the error arose, tracked for lexical context.
        term: Term,
    },
    ResourceBlock {
        /// Term where the error arose, tracked for lexical context.
        term: Term,
        msg: String,
        // TODO(gj): enum for RelatedInformation that has a variant for capturing "other relevant
        // terms" for a particular diagnostic, e.g., for a DuplicateResourceBlock error the
        // already-declared resource block would be relevant info for the error emitted on
        // redeclaration.
    },
    SingletonVariable {
        /// Term<Symbol> where the error arose, tracked for lexical context.
        term: Term,
    },
    UnregisteredClass {
        /// Term<Symbol> where the error arose, tracked for lexical context.
        term: Term,
    },
}

impl ValidationError {
    pub fn with_context(self, kb: &KnowledgeBase) -> PolarError {
        use ValidationError::*;

        let context = match &self {
            // These errors track `term`, from which we calculate the span.
            ResourceBlock { term, .. }
            | SingletonVariable { term, .. }
            | UndefinedRuleCall { term }
            | UnregisteredClass { term, .. } => term.span().zip(kb.get_term_source(term)),

            // These errors track `rule`, from which we calculate the span.
            InvalidRule { rule, .. }
            | InvalidRuleType {
                rule_type: rule, ..
            } => rule.span().zip(kb.get_rule_source(rule)),

            // These errors track `rule_type`, from which we sometimes calculate the span.
            MissingRequiredRule { rule_type } => {
                if rule_type.name.0 == "has_relation" {
                    rule_type.span().zip(kb.get_rule_source(rule_type))
                } else {
                    // TODO(gj): copy source info from the appropriate resource block term for
                    // `has_role()` rule type we create.
                    None
                }
            }
        };

        let context = context.map(|(span, source)| Context {
            range: Range::from_span(&source.src, span),
            source,
        });

        PolarError {
            kind: ErrorKind::Validation(self),
            context,
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidRule { rule, msg } => {
                write!(f, "Invalid rule: {} {}", rule, msg)
            }
            Self::InvalidRuleType { rule_type, msg } => {
                write!(f, "Invalid rule type: {}\n\t{}", rule_type, msg)
            }
            Self::UndefinedRuleCall { term } => {
                write!(f, "Call to undefined rule: {}", term)
            }
            Self::MissingRequiredRule { rule_type } => {
                write!(f, "Missing implementation for required rule {}", rule_type)
            }
            Self::ResourceBlock { msg, .. } => {
                write!(f, "{}", msg)
            }
            Self::SingletonVariable { term } => {
                write!(f, "Singleton variable {term} is unused or undefined; try renaming to _{term} or _", term=term)
            }
            Self::UnregisteredClass { term } => {
                write!(f, "Unregistered class: {}", term)
            }
        }
    }
}

pub fn invalid_state_error<A>(msg: String) -> Result<A, RuntimeError> {
    Err(RuntimeError::InvalidState { msg })
}
