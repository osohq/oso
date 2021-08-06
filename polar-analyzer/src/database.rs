use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
};

use lsp_types::{Position, Range};
use polar_core::kb::KnowledgeBase;

use crate::inspect::{get_rule_information, get_term_information, RuleInfo, TermInfo};

/* Database of information about the code */

#[derive(Debug, Default)]
pub struct SourceMap {
    /// Filename -> Source map
    sources: Arc<RwLock<HashMap<String, Source>>>,
}

impl SourceMap {
    pub fn refresh(&self, kb: &KnowledgeBase, files: Vec<(&str, &str)>) {
        let mut sources = self.sources.write().unwrap();
        let updated_files: HashSet<&str> = files
            .into_iter()
            .map(|(filename, src)| {
                // clear out each source to the default
                sources.insert(filename.to_string(), Source::new(src));
                filename
            })
            .collect();

        for term_info in get_term_information(kb) {
            if let Some(ref f) = term_info.location.0 {
                if updated_files.contains(f.as_str()) {
                    sources.get_mut(f).unwrap().terms.push(term_info);
                }
            }
        }

        for rule_info in get_rule_information(kb) {
            if let Some(ref f) = rule_info.location.0 {
                if updated_files.contains(f.as_str()) {
                    sources.get_mut(f).unwrap().rules.push(rule_info);
                }
            }
        }
    }

    pub fn remove_file(&self, filename: &str) {
        self.sources.write().unwrap().remove(filename);
    }

    pub fn get_term_info(&self, filename: &str) -> Option<Vec<TermInfo>> {
        self.sources
            .read()
            .unwrap()
            .get(filename)
            .map(|source| source.terms.clone())
    }

    pub fn get_rule_info(&self, filename: &str) -> Option<Vec<RuleInfo>> {
        self.sources
            .read()
            .unwrap()
            .get(filename)
            .map(|source| source.rules.clone())
    }

    pub fn location_to_range(&self, filename: &str, start: usize, end: usize) -> Option<Range> {
        self.sources
            .read()
            .unwrap()
            .get(filename)
            .map(|source| Range {
                start: source.offset_to_position(start),
                end: source.offset_to_position(end),
            })
    }

    #[allow(unused)]
    pub fn offset_to_position(&self, filename: &str, offset: usize) -> Option<Position> {
        self.sources
            .read()
            .unwrap()
            .get(filename)
            .map(|source| source.offset_to_position(offset))
    }

    pub fn position_to_offset(&self, filename: &str, position: Position) -> Option<usize> {
        self.sources
            .read()
            .unwrap()
            .get(filename)
            .map(|source| source.position_to_offset(position))
    }

    pub fn get_symbol_at(&self, filename: &str, location: usize) -> Option<TermInfo> {
        self.sources
            .read()
            .unwrap()
            .get(filename)
            .and_then(|source| source.get_symbols_at(location))
    }
}

#[derive(Debug)]
struct Source {
    /// List of line lengths.
    /// Used to convert an LSP `Position` (line/row) into
    /// an offset (as used inside Polar)
    line_lengths: Vec<(usize, usize)>,
    /// List of rules
    rules: Vec<RuleInfo>,
    /// List of terms
    terms: Vec<TermInfo>,
}

impl Source {
    fn new(source: &str) -> Self {
        let mut offset = 0;
        Self {
            rules: Default::default(),
            terms: Default::default(),
            line_lengths: source
                .split('\n')
                .map(str::len)
                .map(|len| {
                    // the offsets of this line are captured as (current_offset, current_offset + len)
                    let start = offset;
                    let end = offset + len;
                    offset = end + 1; // add one for the newline character
                    (start, end)
                })
                .collect(),
        }
    }

    fn position_to_offset(&self, position: Position) -> usize {
        self.line_lengths
            .get(position.line as usize)
            .map(|(start, end)| std::cmp::min(*end, start + position.character as usize))
            .unwrap_or_else(|| self.line_lengths.last().unwrap().1) // we're past the end of the file, take the last offset
    }

    fn offset_to_position(&self, offset: usize) -> Position {
        self.line_lengths
            .binary_search_by(|(start, end)| {
                if offset < *start {
                    Ordering::Greater
                } else if offset > *end {
                    Ordering::Less
                } else {
                    Ordering::Equal
                }
            })
            .map(|idx| Position {
                line: idx as u32,
                character: (offset - self.line_lengths[idx].0) as u32,
            })
            .unwrap_or_else(|_| {
                panic!(
                    "Offset: {} is not found in the document: lines={:#?}",
                    offset, self.line_lengths
                )
            })
    }

    fn get_symbols_at(&self, location: usize) -> Option<TermInfo> {
        let mut symbol = None;
        let mut length = usize::MAX;

        for term in self.terms.iter() {
            let (_, left, right) = term.location;
            if (left..=right).contains(&location) && (right - left) < length {
                symbol = Some(term.clone());
                length = right - left;
            }
        }

        symbol
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;

    #[test]
    fn offset_position_round_trip() {
        let source = Source::new(indoc! {r#"
            f(x) if g(x);

            g(x) if h(x);
        "#});

        let offset_positions = vec![(0, (0, 0)), (8, (0, 8)), (15, (2, 0)), (28, (2, 13))];
        for (offset, (line, char)) in offset_positions {
            let pos = Position::new(line, char);
            assert_eq!(
                offset,
                source.position_to_offset(pos),
                "{:?} -> offset",
                pos
            );
            assert_eq!(pos, source.offset_to_position(offset), "{} -> pos", offset);
            assert_eq!(
                offset,
                source.position_to_offset(source.offset_to_position(offset))
            );
        }
    }
}
