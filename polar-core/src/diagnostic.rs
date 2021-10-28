use std::fmt;

use super::error::PolarError;

pub enum Diagnostic {
    Error(PolarError),
    Warning(String),
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
