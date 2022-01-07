use crate::terms::{Symbol, Term};
use std::collections::HashMap;

#[derive(Default, Debug)]
pub(crate) struct Constants {
    pub symbol_to_term: HashMap<Symbol, Term>,
    id_to_symbol: HashMap<u64, Symbol>,
}

impl Constants {
    pub(crate) fn insert(&mut self, name: Symbol, value: Term, class_id: Option<u64>) {
        self.symbol_to_term.insert(name.clone(), value);
        if let Some(id) = class_id {
            self.id_to_symbol.insert(id, name);
        }
    }

    pub(crate) fn contains_key(&self, name: &Symbol) -> bool {
        self.symbol_to_term.contains_key(name)
    }

    pub(crate) fn get(&self, name: &Symbol) -> Option<&Term> {
        self.symbol_to_term.get(name)
    }

    pub(crate) fn get_symbol_for_class_id(&self, id: u64) -> Option<&Symbol> {
        self.id_to_symbol.get(&id)
    }
}
