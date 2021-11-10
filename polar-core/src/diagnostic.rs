use std::fmt;

use super::error::{ErrorContext, PolarError};
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

    // TODO(gj): ErrorContext -> generic DiagnosticContext type once we add structure to warnings.
    pub fn add_context(&mut self, context: ErrorContext) {
        match self {
            Diagnostic::Error(e) => {
                e.context.replace(context);
            }
            Diagnostic::Warning(_) => (),
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

// Attach context to diagnostics.
//
// TODO(gj): can we attach context to *all* errors here since all errors will be parse-time
// errors and so will have some source context to attach? NOTE(gj): not all -- some errors
// like the absence of an allow rule don't pertain to a particular file or location
// therein.
pub fn set_context_for_diagnostics(kb: &KnowledgeBase, diagnostics: &mut Vec<Diagnostic>) {
    use super::error::{ErrorKind::*, ParseError::*, ValidationError::*};

    for diagnostic in diagnostics {
        let context = match diagnostic {
            Diagnostic::Error(e) => match &e.kind {
                Parse(e) => match e {
                    DuplicateKey {
                        src_id,
                        key: token,
                        loc,
                    }
                    | ExtraToken { src_id, token, loc }
                    | IntegerOverflow { src_id, token, loc }
                    | InvalidFloat { src_id, token, loc }
                    | ReservedWord { src_id, token, loc }
                    | UnrecognizedToken { src_id, token, loc } => {
                        Some(((*loc, loc + token.len()), *src_id))
                    }

                    InvalidTokenCharacter { src_id, loc, .. }
                    | InvalidToken { src_id, loc }
                    | UnrecognizedEOF { src_id, loc } => Some(((*loc, *loc), *src_id)),

                    WrongValueType { src_id, term, .. } => term.span().map(|span| (span, *src_id)),
                },

                Validation(e) => match e {
                    ResourceBlock { ref term, .. }
                    | SingletonVariable { ref term, .. }
                    | UndefinedRuleCall { ref term }
                    | UnregisteredClass { ref term, .. } => term.span().zip(term.get_source_id()),

                    InvalidRule { rule, .. }
                    | InvalidRuleType {
                        rule_type: rule, ..
                    } => rule.parsed_context(),

                    // TODO(gj): copy source info from the appropriate resource block term for
                    // resource-specific rule types we create.
                    MissingRequiredRule { .. } => None,
                },

                Runtime(_) | Operational(_) => None,
            },
            Diagnostic::Warning(_) => None,
        };
        if let Some(((left, _right), src_id)) = context {
            if let Some(source) = kb.sources.get_source(src_id) {
                let (row, column) = crate::lexer::loc_to_pos(&source.src, left);
                diagnostic.add_context(ErrorContext {
                    source,
                    row,
                    column,
                    include_location: false,
                })
            }
        }
    }
}
