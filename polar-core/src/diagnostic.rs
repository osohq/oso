use std::fmt;

use super::error::PolarError;
use super::kb::KnowledgeBase;

#[derive(Debug)]
pub enum Diagnostic {
    Error(PolarError),
    Warning(String),
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

// Attach context to diagnostics.
//
// TODO(gj): can we attach context to *all* errors here since all errors will be parse-time
// errors and so will have some source context to attach? NOTE(gj): not all -- some errors
// like the absence of an allow rule don't pertain to a particular file or location
// therein.
pub fn set_context_for_diagnostics(kb: &KnowledgeBase, diagnostics: &mut Vec<Diagnostic>) {
    for diagnostic in diagnostics {
        if let Diagnostic::Error(e) = diagnostic {
            let source = e.get_source_id().and_then(|id| kb.sources.get_source(id));
            e.set_context(source.as_ref(), None);
        }
    }
}
