mod database;
mod diagnostics;
mod inspect;
pub mod server;

use database::SourceMap;
use diagnostics::UnusedRule;
use inspect::{RuleInfo, TermInfo};
use polar_core::{error::PolarError, polar};

pub use anyhow::Result;
use tracing::{debug, info, warn};

/// Wrapper for the `polar_core::Polar` type.
/// Used as the API interface for all the analytics
pub struct Polar {
    inner: polar::Polar,
    source_map: SourceMap,
}

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
        let res = self
            .inner
            .load(src, Some(filename.to_string()))
            .map_err(|e| {
                // attempt to fall back to the old version _if it was working_
                if let Some(old_src) = old {
                    if self
                        .inner
                        .load(&old_src, Some(filename.to_string()))
                        .is_err()
                    {
                        self.inner.remove_file(filename);
                        let _ = self.inner.load(src, Some(filename.to_string()));
                    }
                }
                e
            });
        match &res {
            Ok(_) => info!("Loaded file {}", filename),
            Err(e) => debug!("Error loading file {}: {}", filename, e),
        }
        // Other than parse errors, we will end up with rules in the KB.
        let kb = self.inner.kb.read().unwrap();
        self.source_map.refresh(&kb, vec![(filename, src)]);
        res
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

    pub fn revalidate(&self, filename: &str) -> Result<(), PolarError> {
        if let Some(old) = self.inner.remove_file(filename) {
            self.inner.load(&old, Some(filename.to_string()))
        } else {
            warn!("Attempting to revalidate a file that doesn't exist");
            Ok(())
        }
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

    pub fn get_files(&self) -> Vec<String> {
        self.source_map.get_files()
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
    tracing_subscriber::fmt::init();

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
