mod context;

pub use context::{Context, Range};

use std::fmt;

use super::error::PolarError;
use super::warning::PolarWarning;

#[derive(Debug)]
pub enum Diagnostic {
    Error(PolarError),
    Warning(PolarWarning),
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
            PolarError::{Parse, Validation},
            ValidationError::{FileLoading, ResourceBlock},
        };
        matches!(
            self,
            Diagnostic::Error(
                Parse(_) | Validation(FileLoading { .. }) | Validation(ResourceBlock { .. }),
            )
        )
    }

    pub fn kind(&self) -> String {
        match self {
            Diagnostic::Error(e) => e.kind(),
            Diagnostic::Warning(w) => w.kind(),
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
