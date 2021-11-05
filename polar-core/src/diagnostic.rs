use std::fmt;

use super::error::PolarError;
use super::warning::Warning;

#[derive(Debug)]
pub enum Diagnostic {
    Error(PolarError),
    Warning(Warning),
}

impl Diagnostic {
    pub fn is_error(&self) -> bool {
        matches!(self, Diagnostic::Error(_))
    }

    pub fn is_parse_error(&self) -> bool {
        use super::error::ErrorKind::Parse;
        matches!(self, Diagnostic::Error(PolarError { kind: Parse(_), .. }))
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Diagnostic::Error(e) => write!(f, "{}", e)?,
            Diagnostic::Warning(w) => write!(f, "{}", w)?,
        }
        Ok(())
    }
}
