use serde::{Deserialize, Serialize};

use std::fmt;

use crate::types::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolarError {
    pub kind: ErrorKind,
    pub formatted: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorKind {
    Parse(ParseError),
    Runtime(RuntimeError),
    Operational(OperationalError),
    Parameter(ParameterError),
}

impl From<ParseError> for PolarError {
    fn from(err: ParseError) -> Self {
        Self {
            formatted: err.to_string(),
            kind: ErrorKind::Parse(err),
        }
    }
}

impl From<RuntimeError> for PolarError {
    fn from(err: RuntimeError) -> Self {
        Self {
            formatted: err.to_string(),
            kind: ErrorKind::Runtime(err),
        }
    }
}

impl From<OperationalError> for PolarError {
    fn from(err: OperationalError) -> Self {
        Self {
            formatted: err.to_string(),
            kind: ErrorKind::Operational(err),
        }
    }
}

impl From<ParameterError> for PolarError {
    fn from(err: ParameterError) -> Self {
        Self {
            formatted: err.to_string(),
            kind: ErrorKind::Parameter(err),
        }
    }
}

pub type PolarResult<T> = std::result::Result<T, PolarError>;

impl std::error::Error for PolarError {}

impl fmt::Display for PolarError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            ErrorKind::Parse(_) => write!(f, "Parse error: ")?,
            ErrorKind::Runtime(_) => write!(f, "Runtime error: ")?,
            ErrorKind::Operational(_) => write!(f, "Operational error: ")?,
            ErrorKind::Parameter(_) => write!(f, "Parameter error: ")?,
        }
        write!(f, "{}", self.formatted)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub source: Source,
    pub row: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParseError {
    IntegerOverflow {
        token: String,
        loc: usize,
        context: Option<ErrorContext>,
    },
    InvalidTokenCharacter {
        token: String,
        c: char,
        loc: usize,
        context: Option<ErrorContext>,
    },
    InvalidToken {
        loc: usize,
        context: Option<ErrorContext>,
    },
    UnrecognizedEOF {
        loc: usize,
        context: Option<ErrorContext>,
    },
    UnrecognizedToken {
        token: String,
        loc: usize,
        context: Option<ErrorContext>,
    },
    ExtraToken {
        token: String,
        loc: usize,
        context: Option<ErrorContext>,
    },
    ReservedWord {
        token: String,
        loc: usize,
        context: Option<ErrorContext>,
    },
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
        let context = match self {
            Self::IntegerOverflow { token, context, .. } => {
                write!(f, "'{}' caused an integer overflow", token.escape_debug())?;
                context
            }
            Self::InvalidTokenCharacter {
                token, c, context, ..
            } => {
                write!(
                    f,
                    "'{}' is not a valid character. Found in {}",
                    c.escape_debug(),
                    token.escape_debug()
                )?;
                context
            }
            Self::InvalidToken { context, .. } => {
                write!(f, "found an unexpected sequence of characters")?;
                context
            }
            Self::UnrecognizedEOF { context, .. } => {
                write!(
                    f,
                    "hit the end of the file unexpectedly. Did you forget a semi-colon"
                )?;
                context
            }
            Self::UnrecognizedToken { token, context, .. } => {
                write!(
                    f,
                    "did not expect to find the token '{}'",
                    token.escape_debug()
                )?;
                context
            }
            Self::ExtraToken { token, context, .. } => {
                write!(
                    f,
                    "did not expect to find the token '{}'",
                    token.escape_debug()
                )?;
                context
            }
            Self::ReservedWord { token, context, .. } => {
                write!(
                    f,
                    "{} is a reserved Polar word and cannot be used here",
                    token.escape_debug()
                )?;
                context
            }
        };
        if let Some(context) = context {
            write!(f, "{}", context)
        } else {
            Ok(())
        }
    }
}

// @TODO: Information about the context of the error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuntimeError {
    Serialization {
        msg: String,
    },
    Unsupported {
        msg: String,
    },
    TypeError {
        msg: String,
        loc: usize,
        context: Option<ErrorContext>,
    },
    UnboundVariable {
        sym: Symbol,
    },
    StackOverflow {
        msg: String,
    },
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Serialization { msg } => write!(f, "Serialization error: {}", msg),
            Self::Unsupported { msg } => write!(f, "Not supported: {}", msg),
            Self::TypeError { msg, loc, context } => {
                write!(f, "Type error: {}", msg)?;
                if let Some(context) = context {
                    write!(f, "{}", context)
                } else {
                    write!(f, " at location {}", loc)
                }
            }
            Self::UnboundVariable { sym } => write!(f, "{} is an unbound variable", sym.0),
            Self::StackOverflow { msg } => write!(f, "Hit a stack limit: {}", msg),
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
