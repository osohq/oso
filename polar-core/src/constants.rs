use crate::terms::{Symbol, Term};
use std::collections::HashMap;

#[derive(Default, Debug)]
pub(crate) struct Constants {
    // Symbol -> Term (populated by *all* constants)
    pub symbol_to_term: HashMap<Symbol, Term>,
    // Symbol -> class_id (populated by class constants)
    class_symbol_to_id: HashMap<Symbol, u64>,
    // class_id -> Symbol (populated by class constants)
    class_id_to_symbol: HashMap<u64, Symbol>,
}

impl Constants {
    pub(crate) fn insert(&mut self, name: Symbol, value: Term) {
        self.symbol_to_term.insert(name, value);
    }

    pub(crate) fn insert_class(&mut self, name: Symbol, value: Term, class_id: u64) {
        self.insert(name.clone(), value);
        self.class_symbol_to_id.insert(name.clone(), class_id);
        self.class_id_to_symbol.insert(class_id, name);
    }

    pub(crate) fn contains_key(&self, name: &Symbol) -> bool {
        self.symbol_to_term.contains_key(name)
    }

    pub(crate) fn get(&self, name: &Symbol) -> Option<&Term> {
        self.symbol_to_term.get(name)
    }

    pub(crate) fn get_class_id_for_symbol(&self, symbol: &Symbol) -> Option<&u64> {
        self.class_symbol_to_id.get(symbol)
    }

    pub(crate) fn get_symbol_for_class_id(&self, id: &u64) -> Option<&Symbol> {
        self.class_id_to_symbol.get(id)
    }
}
