use std::fmt;

use indoc::formatdoc;
use serde::{Deserialize, Serialize};

use super::{
    formatting::source_lines,
    kb::KnowledgeBase,
    rules::Rule,
    sources::Source,
    terms::{Symbol, Term},
};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(into = "FormattedPolarError")]
pub struct PolarError {
    pub kind: ErrorKind,
    pub context: Option<ErrorContext>,
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

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

impl Range {
    pub fn from_span(source: &str, (left, right): (usize, usize)) -> Self {
        let start = Position::from_loc(source, left);
        let end = Position::from_loc(source, right);
        Self { start, end }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Position {
    pub row: usize,
    pub column: usize,
}

impl Position {
    pub fn from_loc(source: &str, loc: usize) -> Self {
        let (row, column) = crate::lexer::loc_to_pos(source, loc);
        Self { row, column }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub source: Source,
    pub range: Range,
}

impl PolarError {
    pub fn set_context(&mut self, source: Option<&Source>, term: Option<&Term>) {
        let span = if let Some(term) = term {
            term.span()
        } else {
            self.span()
        };

        if let (Some(source), Some(span)) = (source, span) {
            let range = Range::from_span(&source.src, span);
            self.context.replace(ErrorContext {
                source: source.clone(),
                range,
            });
        }
    }

    /// Get `(left, right)` span from errors that carry source context.
    fn span(&self) -> Option<(usize, usize)> {
        use {ErrorKind::*, ParseError::*, RuntimeError::*, ValidationError::*};

        match &self.kind {
            Parse(e) => match e {
                DuplicateKey { key: token, loc }
                | ExtraToken { token, loc }
                | IntegerOverflow { token, loc }
                | InvalidFloat { token, loc }
                | ReservedWord { token, loc }
                | UnrecognizedToken { token, loc } => Some((*loc, loc + token.len())),

                InvalidTokenCharacter { loc, .. }
                | InvalidToken { loc }
                | UnrecognizedEOF { loc } => Some((*loc, *loc)),

                WrongValueType { term, .. } => term.span(),
            },

            Validation(e) => match e {
                ResourceBlock { ref term, .. }
                | SingletonVariable { ref term, .. }
                | UndefinedRuleCall { ref term }
                | UnregisteredClass { ref term, .. } => term.span(),

                InvalidRule { rule, .. }
                | InvalidRuleType {
                    rule_type: rule, ..
                } => rule.span(),

                MissingRequiredRule { rule_type } => {
                    if rule_type.name.0 == "has_relation" {
                        rule_type.span()
                    } else {
                        // TODO(gj): copy source info from the appropriate resource block term for
                        // `has_role()` rule type we create.
                        None
                    }
                }
            },

            Runtime(e) => match e {
                Application { term, .. } => term.as_ref().and_then(Term::span),
                ArithmeticError { term }
                | TypeError { term, .. }
                | UnhandledPartial { term, .. }
                | Unsupported { term, .. } => term.span(),
                _ => None,
            },

            Operational(_) => None,
        }
    }
}

impl From<ParseError> for PolarError {
    fn from(err: ParseError) -> Self {
        Self {
            kind: ErrorKind::Parse(err),
            context: None,
        }
    }
}

impl From<RuntimeError> for PolarError {
    fn from(err: RuntimeError) -> Self {
        Self {
            kind: ErrorKind::Runtime(err),
            context: None,
        }
    }
}

impl From<OperationalError> for PolarError {
    fn from(err: OperationalError) -> Self {
        Self {
            kind: ErrorKind::Operational(err),
            context: None,
        }
    }
}

impl From<ValidationError> for PolarError {
    fn from(err: ValidationError) -> Self {
        Self {
            kind: ErrorKind::Validation(err),
            context: None,
        }
    }
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

// TODO(gj): temporary hack
fn pos_to_loc(src: &str, row: usize, column: usize) -> usize {
    let chars_before_row = src.split('\n').take(row).flat_map(|r| r.chars()).count();
    row + chars_before_row + column
}

impl fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Position { row, column } = self.range.start;
        write!(f, " at line {}, column {}", row + 1, column + 1)?;
        if let Some(ref filename) = self.source.filename {
            write!(f, " of file {}", filename)?;
        }
        let loc = pos_to_loc(&self.source.src, row, column);
        let lines = source_lines(&self.source, loc, 0).replace('\n', "\n\t");
        writeln!(f, ":\n\t{}", lines)?;
        Ok(())
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

// @TODO: Information about the context of the error.
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
        /// Simplified term for pretty printing. If it's `None`, fall back to printing `term`.
        simplified: Option<Term>,
        /// Term where the error arose, tracked for lexical context.
        term: Term,
    },
    DataFilteringFieldMissing {
        var_type: String,
        field: String,
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
            Self::UnhandledPartial {
                var,
                simplified,
                term,
            } => {
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
                    expr = simplified.as_ref().unwrap_or(term),
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
            Self::InvalidRegistration { sym, msg } => {
                write!(f, "Invalid attempt to register '{}': {}", sym, msg)
            }
            Self::InvalidState { msg } => write!(f, "Invalid state: {}", msg),
        }
    }
}

// TODO(gj): both of these errors are only constructed/used in the `polar-c-api` crate. We should
// probably move them there.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationalError {
    Serialization {
        msg: String,
    },
    /// Rust panics caught in the `polar-c-api` crate.
    Unknown,
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

impl ValidationError {
    pub fn get_source(&self, kb: &KnowledgeBase) -> Option<Source> {
        use ValidationError::*;

        let src_id = match self {
            ResourceBlock { term, .. }
            | SingletonVariable { term, .. }
            | UndefinedRuleCall { term }
            | UnregisteredClass { term, .. } => term.get_source_id(),

            InvalidRule { rule, .. }
            | InvalidRuleType {
                rule_type: rule, ..
            } => rule.get_source_id(),

            MissingRequiredRule { rule_type } => {
                if rule_type.name.0 == "has_relation" {
                    rule_type.get_source_id()
                } else {
                    // TODO(gj): copy source info from the appropriate resource block term for
                    // `has_role()` rule type we create.
                    None
                }
            }
        };

        src_id.and_then(|id| kb.sources.get_source(id))
    }
}
