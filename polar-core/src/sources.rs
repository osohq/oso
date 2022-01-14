use std::{
    collections::HashMap,
    fmt,
    sync::{Arc, RwLock},
};

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use crate::{formatting::source_lines, lexer::loc_to_pos};

lazy_static! {
    pub(crate) static ref SOURCES: Arc<RwLock<HashMap<u64, Source>>> = Default::default();
}

/// Parsed source context.
#[derive(Clone, Debug)]
pub struct Context {
    pub src_id: u64,
    /// Start location within source.
    pub left: usize,
    /// End location within source.
    pub right: usize,
}

impl fmt::Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.source_position())?;
        if let Some(source) = SOURCES.read().unwrap().get(&self.src_id) {
            let lines = source_lines(source, self.left, 0).replace('\n', "\n\t");
            writeln!(f, ":\n\t{}", lines)?;
        }
        Ok(())
    }
}

impl Context {
    pub(crate) fn new(src_id: u64, left: usize, right: usize) -> Self {
        Self {
            src_id,
            left,
            right,
        }
    }

    pub(crate) fn source_position(&self) -> String {
        let mut f = String::new();
        if let Some(source) = SOURCES.read().unwrap().get(&self.src_id) {
            let (row, column) = loc_to_pos(&source.src, self.left);
            f += &format!(" at line {}, column {}", row + 1, column + 1);
            if let Some(ref filename) = source.filename {
                f += &format!(" of file {}", filename);
            }
        }
        f
    }
}

#[derive(Debug, Clone)]
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

impl SourceInfo {
    pub fn ffi() -> Self {
        Self::Ffi
    }

    pub(crate) fn parser(src_id: u64, left: usize, right: usize) -> Self {
        Self::Parser(Context::new(src_id, left, right))
    }
}

// TODO(gj): `Serialize` makes some `polar-wasm-api` tests easier to write. We could look into
// https://serde.rs/remote-derive.html if we cared to preserve that while removing this impl.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
