use std::collections::HashMap;

use super::counter::Counter;
use super::rules::*;
use super::sources::*;
use super::terms::*;

use std::sync::Arc;

/// A map of bindings: variable name → value. The VM uses a stack internally,
/// but can translate to and from this type.
pub type Bindings = HashMap<Symbol, Term>;

// pub struct ScopeDefinition {
//     name: Symbol,

//     /// Scopes that you can call rules from this scope.
//     included_names: HashSet<Path>,

//     rule_templates: HashMap<Symbol, GenericRule>,
//     // type definitions
// }

#[derive(Default)]
pub struct Scope {
    name: Path,
    constants: Bindings,
    rules: HashMap<Symbol, GenericRule>,
}

impl Scope {
    pub fn new(name: Path) -> Self {
        Self {
            name: name,
            constants: HashMap::new(),
            rules: HashMap::new(),
        }
    }
}

#[derive(Default)]
pub struct KnowledgeBase {
    scopes: HashMap<Symbol, Scope>,
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
            scopes: Default::default(),
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

    /// Generate a new symbol.
    pub fn gensym(&self, prefix: &str) -> Symbol {
        let next = self.gensym_counter.next();
        if prefix == "_" {
            Symbol(format!("_{}", next))
        } else if prefix.starts_with('_') {
            Symbol(format!("{}_{}", prefix, next))
        } else {
            Symbol(format!("_{}_{}", prefix, next))
        }
    }

    /// Define a constant variable. (in the default scope)
    pub fn constant(&mut self, name: Symbol, value: Term) {
        self.scopes
            .entry(sym!("default"))
            .or_insert(Scope::new(sym!("default").into()))
            .constants
            .insert(name, value);
    }

    /// Return true if a constant with the given name has been defined.
    pub fn is_constant(&self, symbol: &Symbol) -> bool {
        self.lookup_constant(symbol.clone().into(), sym!("default").into())
            .is_some()
    }

    pub fn lookup_constant(&self, path: Path, scope: Path) -> Option<&Term> {
        let scope = self.scopes.get(&scope.into_1()).unwrap();
        // .entry(scope.into_1())
        // .or_insert(Scope::new(sym!("default").into()));
        match path.into_2() {
            (name, None) => scope.constants.get(&name),
            (included_scope, Some(name)) => self
                .get_included_scope(scope, included_scope.into())
                .unwrap()
                .constants
                .get(&name),
        }
    }

    /// Get `included` scope w.r.t `base`.
    fn get_included_scope(&self, _base: &Scope, included: Path) -> Option<&Scope> {
        // For now everything is included in everything.
        self.scopes.get(&included.into_1())
    }

    pub fn lookup_rule(&self, path: Path, scope: Path) -> Option<&GenericRule> {
        let scope = self.scopes.get(&scope.into_1()).unwrap();

        match path.into_2() {
            (rule_name, None) => scope.rules.get(&rule_name),
            (included_scope, Some(rule_name)) => self
                .get_included_scope(scope, included_scope.into())
                .unwrap()
                .rules
                .get(&rule_name),
        }
    }

    pub fn add_rule(&mut self, rule: Rule, scope: Path) {
        let scope = self.scopes.get_mut(&scope.into_1()).unwrap();

        let name = rule.name.clone();
        let generic_rule = scope
            .rules
            .entry(name.clone())
            .or_insert_with(|| GenericRule::new(name, vec![]));
        generic_rule.add_rule(Arc::new(rule));
    }

    /// Clear rules from KB, leaving constants in place.
    pub fn clear_rules(&mut self) {
        for scope in self.scopes.iter_mut() {
            scope.1.rules.clear()
        }
        self.sources = Sources::default();
        self.inline_queries.clear();
    }
}
