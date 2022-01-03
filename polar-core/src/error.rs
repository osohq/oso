use std::{fmt, sync::Arc};

use indoc::formatdoc;
use serde::{Deserialize, Serialize};
use strum_macros::AsRefStr;

use super::{
    diagnostic::{Context, Range},
    formatting::to_polar::ToPolarString,
    resource_block::Declaration,
    rules::Rule,
    sources::Source,
    terms::{Operation, Symbol, Term},
};

impl PolarError {
    pub fn kind(&self) -> String {
        use ErrorKind::*;
        match &self.0 {
            Operational(o) => "OperationalError::".to_string() + o.as_ref(),
            Parse(p) => "ParseError::".to_string() + p.as_ref(),
            Runtime(r) => "RuntimeError::".to_string() + r.as_ref(),
            Validation(v) => "ValidationError::".to_string() + v.as_ref(),
        }
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
            kind: other.0,
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

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Parse(e) => write!(f, "{}", e),
            Self::Runtime(e) => write!(f, "{}", e),
            Self::Operational(e) => write!(f, "{}", e),
            Self::Validation(e) => write!(f, "{}", e),
        }
    }
}

// NOTE(gj): `ErrorKind` is a layer of indirection so we can avoid infinite recursion when
// serializing `PolarError` into `FormattedPolarError`, which references the error kind. If
// `PolarError` were the enum (without `ErrorKind`), then `PolarError` would serialize into
// `FormattedPolarError`, which has a field of type `PolarError`... etc. There's probably a better
// way to structure this, but for now this is the path of least resistance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(into = "FormattedPolarError")]
pub struct PolarError(pub ErrorKind);

impl fmt::Display for PolarError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)?;
        if let Some(context) = self.get_context() {
            write!(f, "{}", context)?;
        }
        Ok(())
    }
}

pub type PolarResult<T> = Result<T, PolarError>;

impl std::error::Error for PolarError {}

#[derive(AsRefStr, Clone, Debug, Deserialize, Serialize)]
pub enum ParseError {
    IntegerOverflow {
        source: Arc<Source>,
        token: String,
        loc: usize,
    },
    InvalidTokenCharacter {
        source: Arc<Source>,
        token: String,
        c: char,
        loc: usize,
    },
    InvalidToken {
        source: Arc<Source>,
        loc: usize,
    },
    #[allow(clippy::upper_case_acronyms)]
    UnrecognizedEOF {
        source: Arc<Source>,
        loc: usize,
    },
    UnrecognizedToken {
        source: Arc<Source>,
        token: String,
        loc: usize,
    },
    ExtraToken {
        source: Arc<Source>,
        token: String,
        loc: usize,
    },
    ReservedWord {
        source: Arc<Source>,
        token: String,
        loc: usize,
    },
    InvalidFloat {
        source: Arc<Source>,
        token: String,
        loc: usize,
    },
    WrongValueType {
        source: Arc<Source>,
        loc: usize,
        term: Term,
        expected: String,
    },
    DuplicateKey {
        source: Arc<Source>,
        loc: usize,
        key: String,
    },
}

impl From<ParseError> for PolarError {
    fn from(err: ParseError) -> Self {
        Self(ErrorKind::Parse(err))
    }
}

impl PolarError {
    pub fn get_context(&self) -> Option<Context> {
        use ErrorKind::*;
        use ParseError::*;
        use RuntimeError::*;
        use ValidationError::*;

        let context = match &self.0 {
            Parse(e) => Some(match e {
                // These errors track `loc` (left bound) and `token`, and we calculate right bound
                // as `loc + token.len()`.
                DuplicateKey {
                    key: token,
                    loc,
                    source,
                }
                | ExtraToken { token, loc, source }
                | IntegerOverflow { token, loc, source }
                | InvalidFloat { token, loc, source }
                | ReservedWord { token, loc, source }
                | UnrecognizedToken { token, loc, source } => (source, *loc, loc + token.len()),

                // These errors track `loc` and only pertain to a single character, so right bound
                // of span is also `loc`.
                InvalidTokenCharacter { loc, source, .. }
                | InvalidToken { loc, source }
                | UnrecognizedEOF { loc, source } => (source, *loc, *loc),

                // These errors track `term`, from which we calculate the span.
                WrongValueType { term, .. } => {
                    term.parsed_source_info().expect("always from parser")
                }
            }),

            Runtime(e) => match e {
                // These errors sometimes track `term`, from which we derive context.
                Application { term, .. } => term.as_ref().and_then(Term::parsed_source_info),

                // These errors track `term`, from which we derive the context.
                ArithmeticError { term }
                | TypeError { term, .. }
                | UnhandledPartial { term, .. }
                | Unsupported { term, .. } => term.parsed_source_info(),

                // These errors never have context.
                StackOverflow { .. }
                | QueryTimeout { .. }
                | IncompatibleBindings { .. }
                | DataFilteringFieldMissing { .. }
                | DataFilteringUnsupportedOp { .. }
                | InvalidRegistration { .. }
                | QueryForUndefinedRule { .. }
                | MultipleLoadError => None,
            },

            Validation(e) => match e {
                // These errors track `term`, from which we calculate the span.
                ResourceBlock { term, .. }
                | SingletonVariable { term, .. }
                | UndefinedRuleCall { term }
                | DuplicateResourceBlockDeclaration {
                    declaration: term, ..
                }
                | UnregisteredClass { term, .. } => term.parsed_source_info(),

                // These errors track `rule`, from which we calculate the span.
                InvalidRule { rule, .. }
                | InvalidRuleType {
                    rule_type: rule, ..
                } => rule.parsed_source_info(),

                // These errors track `rule_type`, from which we sometimes calculate the span.
                MissingRequiredRule { rule_type } => {
                    if rule_type.name.0 == "has_relation" {
                        rule_type.parsed_source_info()
                    } else {
                        // TODO(gj): copy source info from the appropriate resource block term for
                        // `has_role()` rule type we create.
                        None
                    }
                }

                // These errors always pertain to a specific file but not to a specific place therein.
                FileLoading { source, .. } => Some((source, 0, 0)),
            },

            Operational(_) => None,
        };

        context.map(|(source, left, right)| Context {
            range: Range::from_span(&source.src, (left, right)),
            source: source.clone(),
        })
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

#[derive(AsRefStr, Clone, Debug, Deserialize, Serialize)]
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
    MultipleLoadError,
    /// The user queried for an undefined rule. This is the runtime analogue of
    /// `ValidationError::UndefinedRuleCall`.
    QueryForUndefinedRule {
        name: String,
    },
}

impl From<RuntimeError> for PolarError {
    fn from(err: RuntimeError) -> Self {
        Self(ErrorKind::Runtime(err))
    }
}

impl RuntimeError {
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
                    r#"Unsupported operation:
                        {:?}/{}
                    in the expression:
                        {}
                    in a data filtering query.

                    This operation is not currently supported for data filtering.
                    For more information please refer to our documentation:
                        https://docs.osohq.com/guides/data_filtering.html
                    "#,
                    operation.operator,
                    operation.args.len(),
                    operation.to_polar()
                );
                write!(f, "{}", msg)
            }
            Self::InvalidRegistration { sym, msg } => {
                write!(f, "Invalid attempt to register '{}': {}", sym, msg)
            }
            Self::MultipleLoadError => write!(f, "Cannot load additional Polar code -- all Polar code must be loaded at the same time."),
            Self::QueryForUndefinedRule { name } => write!(f, "Query for undefined rule `{}`", name),
        }
    }
}

#[derive(AsRefStr, Clone, Debug, Deserialize, Serialize)]
pub enum OperationalError {
    /// An invariant has been broken internally.
    InvalidState { msg: String },
    /// Serialization errors in the `polar-c-api` crate.
    Serialization { msg: String },
    /// Rust panics caught in the `polar-c-api` crate.
    Unknown,
}

impl From<OperationalError> for PolarError {
    fn from(err: OperationalError) -> Self {
        Self(ErrorKind::Operational(err))
    }
}

impl fmt::Display for OperationalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidState { msg } => write!(f, "Invalid state: {}", msg),
            Self::Serialization { msg } => write!(f, "Serialization error: {}", msg),
            Self::Unknown => write!(
                f,
                "We hit an unexpected error.\n\
                Please submit a bug report at <https://github.com/osohq/oso/issues>"
            ),
        }
    }
}

#[derive(AsRefStr, Clone, Debug, Deserialize, Serialize)]
pub enum ValidationError {
    FileLoading {
        source: Arc<Source>,
        msg: String,
    },
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
    /// The policy contains a call to an undefined rule. This is the validation analogue of
    /// `RuntimeError::QueryForUndefinedRule`.
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
    DuplicateResourceBlockDeclaration {
        /// Term<Symbol> where the error arose.
        resource: Term,
        /// Term<String> where the error arose, tracked for lexical context.
        declaration: Term,
        existing: Declaration,
        new: Declaration,
    },
}

impl From<ValidationError> for PolarError {
    fn from(err: ValidationError) -> Self {
        Self(ErrorKind::Validation(err))
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::FileLoading { msg, .. } => write!(f, "Problem loading file: {}", msg),
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
            Self::DuplicateResourceBlockDeclaration {
                resource,
                declaration,
                existing,
                new,
            } => {
                write!(
                    f,
                    "Cannot overwrite existing {} declaration {} in resource {} with {}",
                    existing, declaration, resource, new
                )
            }
        }
    }
}

pub(crate) fn invalid_state<T, U>(msg: T) -> PolarResult<U>
where
    T: AsRef<str>,
{
    let msg = msg.as_ref().into();
    Err(OperationalError::InvalidState { msg }.into())
}
