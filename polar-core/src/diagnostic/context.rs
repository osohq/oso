use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{formatting::source_lines, sources::Source};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

impl Range {
    pub fn from_span(source: &str, (left, right): (usize, usize)) -> Self {
        let start = Position::from_loc(source, left);
        let end = Position::from_loc(source, right);
        Self { start, end }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Position {
    pub row: usize,
    pub column: usize,
}

impl Position {
    pub fn from_loc(source: &str, loc: usize) -> Self {
        let (row, column) = crate::lexer::loc_to_pos(source, loc);
        Self { row, column }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Context {
    pub source: Source,
    pub range: Range,
}

// TODO(gj): temporary hack -- this won't be necessary once `formatting::source_lines` takes a
// `Range` instead of a single `usize` (`loc`).
fn pos_to_loc(src: &str, row: usize, column: usize) -> usize {
    let chars_before_row = src.split('\n').take(row).flat_map(|r| r.chars()).count();
    row + chars_before_row + column
}

impl fmt::Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Position { row, column } = self.range.start;
        write!(f, " at line {}, column {}", row + 1, column + 1)?;
        if let Some(ref filename) = self.source.filename {
            write!(f, " of file {}", filename)?;
        }
        let loc = pos_to_loc(&self.source.src, row, column);
        let lines = source_lines(&self.source, loc, 0).replace('\n', "\n\t");
        writeln!(f, ":\n\t{}", lines)?;
        Ok(())
    }
}

impl Context {
    pub fn source_file_and_line(&self) -> String {
        let Position { row, .. } = self.range.start;
        if let Some(filename) = &self.source.filename {
            format!("{}:{}", filename, row)
        } else {
            format!(":{}", row)
        }
    }
}
