use std::collections::HashMap;

use crate::error::PolarResult;

pub use super::bindings::Bindings;
use super::counter::Counter;
use super::rules::*;
use super::sources::*;
use super::terms::*;

/// A map of bindings: variable name → value. The VM uses a stack internally,
/// but can translate to and from this type.

#[derive(Default)]
pub struct KnowledgeBase {
    pub constants: Bindings,

    /// Map from loaded files to the source ID
    pub loaded_files: HashMap<String, u64>,
    /// Map from source code loaded to the filename it was loaded as
    pub loaded_content: HashMap<String, String>,

    pub rules: HashMap<Symbol, GenericRule>,
    pub sources: Sources,
    /// For symbols returned from gensym.
    gensym_counter: Counter,
    /// For call IDs, instance IDs, symbols, etc.
    id_counter: Counter,
    pub inline_queries: Vec<Term>,
}

impl KnowledgeBase {
    pub fn new() -> Self {
        Self {
            constants: HashMap::new(),
            loaded_files: Default::default(),
            loaded_content: Default::default(),
            rules: HashMap::new(),
            sources: Sources::default(),
            id_counter: Counter::default(),
            gensym_counter: Counter::default(),
            inline_queries: vec![],
        }
    }

    /// Return a monotonically increasing integer ID.
    ///
    /// Wraps around at 52 bits of precision so that it can be safely
    /// coerced to an IEEE-754 double-float (f64).
    pub fn new_id(&self) -> u64 {
        self.id_counter.next()
    }

    pub fn id_counter(&self) -> Counter {
        self.id_counter.clone()
    }

    /// Generate a temporary variable prefix from a variable name.
    pub fn temp_prefix(name: &str) -> String {
        match name {
            "_" => String::from(name),
            _ => format!("_{}_", name),
        }
    }

    /// Generate a new symbol.
    pub fn gensym(&self, prefix: &str) -> Symbol {
        let next = self.gensym_counter.next();
        Symbol(format!("{}{}", Self::temp_prefix(prefix), next))
    }

    /// Add a generic rule to the knowledge base.
    #[cfg(test)]
    pub fn add_generic_rule(&mut self, rule: GenericRule) {
        self.rules.insert(rule.name.clone(), rule);
    }

    /// Define a constant variable.
    pub fn constant(&mut self, name: Symbol, value: Term) {
        self.constants.insert(name, value);
    }

    /// Return true if a constant with the given name has been defined.
    pub fn is_constant(&self, name: &Symbol) -> bool {
        self.constants.contains_key(name)
    }

    pub fn add_source(&mut self, source: Source) -> PolarResult<u64> {
        let src_id = self.new_id();
        if let Some(ref filename) = source.filename {
            self.check_file(&source.src, &filename)?;
            self.loaded_content
                .insert(source.src.clone(), filename.to_string());
            self.loaded_files.insert(filename.to_string(), src_id);
        }
        self.sources.add_source(source, src_id);
        Ok(src_id)
    }

    pub fn clear_rules(&mut self) {
        self.rules.clear();
        self.sources = Sources::default();
        self.inline_queries.clear();
        self.loaded_content.clear();
        self.loaded_files.clear();
    }

    pub fn remove_file(&mut self, filename: &str) -> Option<String> {
        self.loaded_files
            .get(filename)
            .cloned()
            .map(|src_id| self.remove_source(Some(filename.to_string()), src_id))
    }

    pub fn remove_source(&mut self, filename: Option<String>, source_id: u64) -> String {
        // remove from rules
        self.rules.retain(|_, gr| {
            let to_remove: Vec<u64> = gr.rules.iter().filter_map(|(idx, rule)| {
                if matches!(rule.source_info, SourceInfo::Parser { src_id, ..} if src_id == source_id) {
                    Some(*idx)
                } else {
                    None
                }
            }).collect();

            for idx in to_remove {
                gr.remove_rule(idx);
            }
            !gr.rules.is_empty()
        });

        // remove from sources
        let source = self
            .sources
            .remove_source(source_id)
            .expect("source doesn't exist in KB");

        assert_eq!(source.filename, filename);

        // remove queries
        self.inline_queries
            .retain(|q| q.get_source_id() != Some(source_id));

        // remove from files
        if let Some(filename) = filename {
            self.loaded_files.remove(&filename);
            self.loaded_content.retain(|_, f| f != &filename);
        }
        source.src
    }

    fn check_file(&self, src: &str, filename: &str) -> PolarResult<()> {
        match (
            self.loaded_content.get(src),
            self.loaded_files.get(filename).is_some(),
        ) {
            (Some(other_file), true) if other_file == filename => {
                return Err(error::RuntimeError::FileLoading {
                    msg: format!("File {} has already been loaded.", filename),
                }
                .into())
            }
            (_, true) => {
                return Err(error::RuntimeError::FileLoading {
                    msg: format!(
                        "A file with the name {}, but different contents has already been loaded.",
                        filename
                    ),
                }
                .into());
            }
            (Some(other_file), _) => {
                return Err(error::RuntimeError::FileLoading {
                    msg: format!(
                        "A file with the same contents as {} named {} has already been loaded.",
                        filename, other_file
                    ),
                }
                .into());
            }
            _ => {}
        }
        Ok(())
    }
}
