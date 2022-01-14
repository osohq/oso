use std::fmt;

use super::{error::PolarError, sources::Context, warning::PolarWarning};

#[derive(Debug)]
pub enum Diagnostic {
    Error(PolarError),
    Warning(PolarWarning),
}

impl From<PolarError> for Diagnostic {
    fn from(err: PolarError) -> Self {
        Self::Error(err)
    }
}

impl Diagnostic {
    pub fn is_error(&self) -> bool {
        matches!(self, Diagnostic::Error(_))
    }

    /// Unrecoverable diagnostics might lead to additional diagnostics that obscure the root issue.
    ///
    /// E.g., a `ResourceBlock` error for an invalid `relations` declaration that will cause a
    /// second `ResourceBlock` error when rewriting a shorthand rule involving the relation.
    pub fn is_unrecoverable(&self) -> bool {
        use super::error::{
            ErrorKind::{Parse, Validation},
            ValidationError::{FileLoading, ResourceBlock},
        };
        matches!(
            self,
            Diagnostic::Error(PolarError(
                Parse(_) | Validation(FileLoading { .. }) | Validation(ResourceBlock { .. }),
            ))
        )
    }

    pub fn kind(&self) -> String {
        match self {
            Diagnostic::Error(e) => e.kind(),
            Diagnostic::Warning(w) => w.kind(),
        }
    }

    pub fn get_context(&self) -> Option<Context> {
        match self {
            Diagnostic::Error(e) => e.get_context(),
            Diagnostic::Warning(w) => w.get_context(),
        }
    }
}

#[cfg(test)]
impl Diagnostic {
    pub fn unwrap_error(self) -> PolarError {
        match self {
            Diagnostic::Error(e) => e,
            _ => panic!(),
        }
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
