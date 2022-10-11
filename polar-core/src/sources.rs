use std::fmt::Write;
use std::{fmt, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::{formatting::source_lines, lexer::loc_to_pos};

/// Parsed source context.
#[derive(Clone)]
pub struct Context {
    pub source: Arc<Source>,
    /// Start location within `source`.
    pub left: usize,
    /// End location within `source`.
    pub right: usize,
}

impl fmt::Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.source_position())?;
        let lines = source_lines(&self.source, self.left, 0).replace('\n', "\n\t");
        writeln!(f, ":\n\t{}", lines)?;
        Ok(())
    }
}

impl Context {
    pub(crate) fn new(source: Arc<Source>, left: usize, right: usize) -> Self {
        Self {
            source,
            left,
            right,
        }
    }

    pub(crate) fn source_position(&self) -> String {
        let mut f = String::new();
        let (row, column) = loc_to_pos(&self.source.src, self.left);
        write!(f, " at line {}, column {}", row + 1, column + 1).unwrap();
        if let Some(ref filename) = self.source.filename {
            write!(f, " of file {}", filename).unwrap();
        }
        f
    }
}

#[derive(Clone)]
pub enum SourceInfo {
    // From the parser
    Parser(Context),

    /// Created as a temporary variable
    TemporaryVariable,

    /// From an FFI call
    Ffi,

    /// Created for a test
    Test,
}

impl fmt::Debug for SourceInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Ignore inner `Context` when `Debug`-formatting `SourceInfo::Parser`.
            Self::Parser(_) => f.write_str("SourceInfo::Parser"),
            Self::TemporaryVariable => f.write_str("SourceInfo::TemporaryVariable"),
            Self::Ffi => f.write_str("SourceInfo::Ffi"),
            Self::Test => f.write_str("SourceInfo::Test"),
        }
    }
}

impl SourceInfo {
    pub fn ffi() -> Self {
        Self::Ffi
    }

    pub(crate) fn parser(source: Arc<Source>, left: usize, right: usize) -> Self {
        Self::Parser(Context::new(source, left, right))
    }
}

// TODO(gj): `Serialize` makes some `polar-wasm-api` tests easier to write. We could look into
// https://serde.rs/remote-derive.html if we cared to preserve that while removing this impl.
#[derive(PartialEq, Eq, Serialize, Deserialize)]
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
