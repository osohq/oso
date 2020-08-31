use serde::{Deserialize, Serialize};

use std::fmt;

use crate::types::*;

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
    Parameter(ParameterError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub source: Source,
    pub row: usize,
    pub column: usize,
}

impl PolarError {
    pub fn set_context(mut self, source: Option<&Source>, term: Option<&Term>) -> Self {
        match (&self.kind, source, term) {
            (ErrorKind::Parse(e), Some(source), _) => match e {
                ParseError::IntegerOverflow { loc, .. }
                | ParseError::InvalidTokenCharacter { loc, .. }
                | ParseError::InvalidToken { loc, .. }
                | ParseError::UnrecognizedEOF { loc }
                | ParseError::UnrecognizedToken { loc, .. }
                | ParseError::ExtraToken { loc, .. } => {
                    let (row, column) = crate::lexer::loc_to_pos(&source.src, *loc);
                    self.context.replace(ErrorContext {
                        source: source.clone(),
                        row,
                        column,
                    });
                }
                _ => {}
            },
            (_, Some(source), Some(term)) => {
                let (row, column) = crate::lexer::loc_to_pos(&source.src, term.offset());
                self.context.replace(ErrorContext {
                    source: source.clone(),
                    row,
                    column,
                });
            }
            _ => {}
        }
        self
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

impl From<ParameterError> for PolarError {
    fn from(err: ParameterError) -> Self {
        Self {
            kind: ErrorKind::Parameter(err),
            context: None,
        }
    }
}

pub type PolarResult<T> = std::result::Result<T, PolarError>;

impl std::error::Error for PolarError {}

impl fmt::Display for PolarError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.kind {
            ErrorKind::Parse(e) => write!(f, "{}", e)?,
            ErrorKind::Runtime(e) => write!(f, "{}", e)?,
            ErrorKind::Operational(e) => write!(f, "{}", e)?,
            ErrorKind::Parameter(e) => write!(f, "{}", e)?,
        }
        if let Some(ref context) = self.context {
            write!(f, "{}", context)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParseError {
    IntegerOverflow { token: String, loc: usize },
    InvalidTokenCharacter { token: String, c: char, loc: usize },
    InvalidToken { loc: usize },
    UnrecognizedEOF { loc: usize },
    UnrecognizedToken { token: String, loc: usize },
    ExtraToken { token: String, loc: usize },
    ReservedWord { token: String, loc: usize },
    InvalidFloat { token: String, loc: usize },
}

impl fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, " at line {}, column {}", self.row + 1, self.column + 1)?;
        if let Some(ref filename) = self.source.filename {
            write!(f, " in file {}", filename)?;
        }
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
        }
    }
}

// @TODO: Information about the context of the error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuntimeError {
    ArithmeticError {
        msg: String,
    },
    Serialization {
        msg: String,
    },
    Unsupported {
        msg: String,
    },
    TypeError {
        msg: String,
        stack_trace: Option<String>,
    },
    UnboundVariable {
        sym: Symbol,
    },
    StackOverflow {
        msg: String,
    },
    QueryTimeout {
        msg: String,
    },
    Application {
        msg: String,
        stack_trace: Option<String>,
    },
    FileLoading {
        msg: String,
    },
}

impl RuntimeError {
    pub fn add_stack_trace(&mut self, vm: &crate::vm::PolarVirtualMachine) {
        match self {
            Self::Application { stack_trace, .. } | Self::TypeError { stack_trace, .. } => {
                *stack_trace = Some(vm.stack_trace())
            }
            _ => {}
        }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ArithmeticError { msg } => write!(f, "Arithmetic error: {}", msg),
            Self::Serialization { msg } => write!(f, "Serialization error: {}", msg),
            Self::Unsupported { msg } => write!(f, "Not supported: {}", msg),
            Self::TypeError { msg, stack_trace } => {
                if let Some(stack_trace) = stack_trace {
                    writeln!(f, "{}", stack_trace)?;
                }
                write!(f, "Type error: {}", msg)
            }
            Self::UnboundVariable { sym } => write!(f, "{} is an unbound variable", sym.0),
            Self::StackOverflow { msg } => write!(f, "Hit a stack limit: {}", msg),
            Self::QueryTimeout { msg } => write!(f, "Query timeout: {}", msg),
            Self::Application { msg, stack_trace } => {
                if let Some(stack_trace) = stack_trace {
                    writeln!(f, "{}", stack_trace)?;
                }
                write!(f, "Application error: {}", msg)
            }
            Self::FileLoading { msg } => write!(f, "Problem loading file: {}", msg),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationalError {
    Unimplemented(String),
    Unknown,
}

impl fmt::Display for OperationalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Unimplemented(s) => write!(f, "{} is not yet implemented", s),
            Self::Unknown => write!(f, "we hit an error we do not know how to handle or did not expect. Please submit a bug"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Parameter passed to FFI lib function is invalid.
pub struct ParameterError(pub String);

impl fmt::Display for ParameterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid parameter used in FFI function: {}", self.0)
    }
}
