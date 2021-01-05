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

pub struct Scope {
    name: Symbol,
    constants: Bindings,
    rules: HashMap<Symbol, GenericRule>,
}

impl Scope {
    pub fn new(name: Symbol) -> Self {
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
        let mut scopes = HashMap::new();
        scopes.insert(sym!("default"), Scope::new(sym!("default")));
        Self {
            scopes: scopes,
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
        // All constants are defined on the default scope; if default scope doesn't exist, add it
        self.scopes
            .entry(sym!("default"))
            .or_insert(Scope::new(sym!("default").into()))
            .constants
            .insert(name, value);
    }

    /// Return true if a constant with the given name has been defined.
    pub fn is_constant(&self, symbol: &Symbol) -> bool {
        self.lookup_constant(Path::with_name(symbol.clone()), &sym!("default"))
            .is_some()
    }

    pub fn lookup_constant(&self, const_path: Path, scope: &Symbol) -> Option<&Term> {
        // lookup scope by path; return `None` if scope doesn't exist
        self.scopes.get(&scope).and_then(|scope| {
            match (const_path.scope(), const_path.name()) {
                // if there is no included scope, get the constant from the current scope
                (None, const_name) => scope.constants.get(&const_name),
                // if there is an included scope, check that the scope is included and get the constant from the included scope
                (Some(included_scope), const_name) => self
                    .get_included_scope(scope, included_scope)
                    .and_then(|scope| scope.constants.get(&const_name)),
            }
        })
    }

    /// Get `included` scope w.r.t `base`.
    fn get_included_scope(&self, _base: &Scope, included: &Symbol) -> Option<&Scope> {
        // For now everything is included in everything.
        self.scopes.get(included)
    }

    pub fn lookup_rule(&self, rule_path: Path, in_scope: &Symbol) -> Option<&GenericRule> {
        // lookup scope by path; return `None` if scope doesn't exist
        self.scopes.get(&in_scope).and_then(|scope| {
            match (rule_path.scope(), rule_path.name()) {
                // if there is no included scope, get the rule from the current scope
                (None, name) => scope.rules.get(&name),
                // if there is a scope name, check that the scope is included and get the rule from the included scope
                (Some(included_scope), name) => self
                    .get_included_scope(scope, included_scope)
                    .and_then(|scope| scope.rules.get(&name)),
            }
        })
    }

    pub fn add_rule(&mut self, rule: Rule, scope: Symbol) {
        // lookup scope by path; panic if scope doesn't exist
        let scope = self
            .scopes
            .entry(scope.clone())
            .or_insert_with(|| Scope::new(scope));

        let name = rule.name.clone();
        let generic_rule = scope
            .rules
            .entry(name.clone())
            .or_insert_with(|| GenericRule::new(name, vec![]));
        generic_rule.add_rule(Arc::new(rule));
    }

    /// Clear rules from KB, leaving constants in place.
    pub fn clear_rules(&mut self) {
        for (_, scope) in self.scopes.iter_mut() {
            scope.rules.clear()
        }
        self.sources = Sources::default();
        self.inline_queries.clear();
    }
}
