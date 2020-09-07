use super::numerics::MOST_POSITIVE_EXACT_FLOAT;
use super::rules::*;
use super::sources::*;
use super::terms::*;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// A map of bindings: variable name → value. The VM uses a stack internally,
/// but can translate to and from this type.
pub type Bindings = HashMap<Symbol, Term>;

#[derive(Clone)]
pub enum Type {
    Class { name: Symbol },
}

#[derive(Default)]
pub struct KnowledgeBase {
    pub constants: Bindings,
    pub types: HashMap<Symbol, Type>,
    pub rules: HashMap<Symbol, GenericRule>,
    pub sources: Sources,
    /// For symbols returned from gensym.
    gensym_counter: AtomicU64,
    /// For call IDs, instance IDs, symbols, etc.
    id_counter: AtomicU64,
    pub inline_queries: Vec<Term>,
}

const MAX_ID: u64 = (MOST_POSITIVE_EXACT_FLOAT - 1) as u64;

impl KnowledgeBase {
    pub fn new() -> Self {
        Self {
            constants: HashMap::new(),
            types: HashMap::new(),
            rules: HashMap::new(),
            sources: Sources::default(),
            id_counter: AtomicU64::new(1),
            gensym_counter: AtomicU64::new(1),
            inline_queries: vec![],
        }
    }

    /// Return a monotonically increasing integer ID.
    ///
    /// Wraps around at 52 bits of precision so that it can be safely
    /// coerced to an IEEE-754 double-float (f64).
    pub fn new_id(&self) -> u64 {
        if self
            .id_counter
            .compare_and_swap(MAX_ID, 1, Ordering::SeqCst)
            == MAX_ID
        {
            MAX_ID
        } else {
            self.id_counter.fetch_add(1, Ordering::SeqCst)
        }
    }

    /// Generate a new symbol.
    pub fn gensym(&self, prefix: &str) -> Symbol {
        let next = self.gensym_counter.fetch_add(1, Ordering::SeqCst);
        if prefix == "_" {
            Symbol(format!("_{}", next))
        } else if prefix.starts_with('_') {
            Symbol(format!("{}_{}", prefix, next))
        } else {
            Symbol(format!("_{}_{}", prefix, next))
        }
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
}

#[test]
fn test_id_wrapping() {
    let kb = KnowledgeBase::new();
    kb.id_counter.store(MAX_ID - 1, Ordering::SeqCst);
    assert_eq!(MAX_ID - 1, kb.new_id());
    assert_eq!(MAX_ID, kb.new_id());
    assert_eq!(1, kb.new_id());
    assert_eq!(2, kb.new_id());
}
