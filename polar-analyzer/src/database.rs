use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
};

use polar_core::kb::KnowledgeBase;

use crate::inspect::{get_rule_information, get_term_information, RuleInfo, TermInfo};

/* Database of information about the code */

#[derive(Debug, Default)]
pub struct SourceMap {
    /// File -> Source map
    sources: Arc<RwLock<HashMap<String, Source>>>,
}

impl SourceMap {
    pub fn refresh(&self, kb: &KnowledgeBase, files: Vec<&str>) {
        let mut sources = self.sources.write().unwrap();
        let updated_files: HashSet<&str> = files
            .into_iter()
            .inspect(|f| {
                // clear out each source to the default
                sources.insert(f.to_string(), Source::default());
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

    pub fn get_term_info(&self, filename: &str) -> Vec<TermInfo> {
        self.sources
            .read()
            .unwrap()
            .get(filename)
            .map(|source| source.terms.clone())
            .unwrap_or_default()
    }

    pub fn get_rule_info(&self, filename: &str) -> Vec<RuleInfo> {
        self.sources
            .read()
            .unwrap()
            .get(filename)
            .map(|source| source.rules.clone())
            .unwrap_or_default()
    }

    pub fn get_symbol_at(&self, filename: &str, location: usize) -> Option<TermInfo> {
        self.sources
            .read()
            .unwrap()
            .get(filename)
            .and_then(|source| source.get_symbols_at(location))
    }
}

#[derive(Debug, Default)]
struct Source {
    // List of rules
    rules: Vec<RuleInfo>,
    // List of terms
    terms: Vec<TermInfo>,
}

impl Source {
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
