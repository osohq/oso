use std::rc::Rc;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum SourceInfo {
    // TODO(gj): why is this not just `Parser(Context)`?
    //
    // From the parser
    Parser {
        source: Rc<Source>,

        /// Location of the term within the source map
        left: usize,
        right: usize,
    },

    /// Created as a temporary variable
    TemporaryVariable,

    /// From an FFI call
    Ffi,

    /// Created for a test
    Test,
}

impl SourceInfo {
    pub fn ffi() -> Self {
        Self::Ffi
    }
}

// TODO(gj): why is this Serialize? At a minimum I don't think we need to serialize the full source
// text.
#[derive(Debug, Serialize, Deserialize)]
pub struct Source {
    pub filename: Option<String>,
    pub src: String,
}

impl Source {
    pub fn new<T: AsRef<str>>(src: T) -> Self {
        Self {
            filename: None,
            src: src.as_ref().into(),
        }
    }

    pub fn new_with_name<T: AsRef<str>, U: AsRef<str>>(filename: T, src: U) -> Self {
        Self {
            filename: Some(filename.as_ref().into()),
            src: src.as_ref().into(),
        }
    }
}
