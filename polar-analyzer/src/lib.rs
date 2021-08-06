mod database;
mod diagnostics;
mod inspect;
pub mod server;

use database::SourceMap;
use diagnostics::UnusedRule;
use inspect::{RuleInfo, TermInfo};
use polar_core::{error::PolarError, polar};

/// Wrapper for the `polar_core::Polar` type.
/// Used as the API interface for all the analytics
pub struct Polar {
    inner: polar::Polar,
    source_map: SourceMap,
}

pub use anyhow::Result;

impl Default for Polar {
    fn default() -> Self {
        Polar::new()
    }
}

impl Polar {
    pub fn new() -> Self {
        let inner = polar::Polar::new();
        // let _ = inner.enable_roles();
        Self {
            inner,
            source_map: Default::default(),
        }
    }

    pub fn enable_roles(&self) {
        // swallowing errors for now since we can't actually validate
        // anything works yet
        let _ = self.inner.enable_roles();
    }

    /// Loads a file into the knowledge base.
    ///
    /// In comparison to the `Polar` in the core, this
    /// will first remove the file.
    pub fn load(&self, src: &str, filename: &str) -> Result<(), PolarError> {
        let old = self.inner.remove_file(filename);
        self.inner
            .load(src, Some(filename.to_string()))
            .map_err(|e| {
                if let Some(old_src) = old {
                    self.inner
                        .load(&old_src, Some(filename.to_string()))
                        .expect("failed to reload old policy after new policy loading failed");
                }
                e
            })?;
        let kb = self.inner.kb.read().unwrap();
        self.source_map.refresh(&kb, vec![(filename, src)]);
        Ok(())
    }

    pub fn rename(&self, old_filename: &str, new_filename: &str) -> Result<(), PolarError> {
        if let Some(old) = self.inner.remove_file(old_filename) {
            self.source_map.remove_file(old_filename);
            self.load(&old, new_filename)
        } else {
            Ok(())
        }
    }

    pub fn delete(&self, filename: &str) {
        self.source_map.remove_file(filename);
        let _old = self.inner.remove_file(filename);
    }

    pub fn clear_rules(&self) {
        self.inner.clear_rules()
    }

    pub fn get_rule_info(&self, filename: &str) -> Vec<RuleInfo> {
        self.source_map.get_rule_info(filename).unwrap_or_default()
    }

    pub fn get_term_info(&self, filename: &str) -> Vec<TermInfo> {
        self.source_map.get_term_info(filename).unwrap_or_default()
    }

    pub fn get_unused_rules(&self, filename: &str) -> Vec<UnusedRule> {
        let kb = self.inner.kb.read().unwrap();
        diagnostics::find_missing_rules(&kb, filename)
    }

    pub fn get_symbol_at(&self, filename: &str, location: usize) -> Option<TermInfo> {
        self.source_map.get_symbol_at(filename, location)
    }

    #[cfg(test)]
    pub(crate) fn with_kb<F, R>(&self, f: F) -> R
    where
        F: Fn(&polar_core::kb::KnowledgeBase) -> R,
    {
        let kb = self.inner.kb.read().unwrap();
        f(&kb)
    }
}

pub fn run_polar_analyzer(inner: polar::Polar, port: u32) {
    let source_map = SourceMap::default();
    {
        let kb = inner.kb.read().unwrap();
        let files = kb
            .sources
            .sources
            .iter()
            .filter_map(|(_, source)| {
                source
                    .filename
                    .as_ref()
                    .map(|f| (f.as_str(), source.src.as_str()))
            })
            .collect();
        source_map.refresh(&kb, files);
    }

    let polar = Polar { inner, source_map };
    let _res = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(server::run_tcp_server(Some(polar), port));
}
