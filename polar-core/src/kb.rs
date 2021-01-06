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
//     // included_names: HashSet<Path>,
//     // type definitions
// }

pub struct Scope {
    name: Symbol,
    constants: Bindings,
    rule_templates: HashMap<Symbol, Vec<Rule>>,
    rules: HashMap<Symbol, GenericRule>,
}

impl Scope {
    pub fn new(name: Symbol) -> Self {
        Self {
            name: name,
            constants: HashMap::new(),
            rule_templates: HashMap::new(),
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

    pub fn lookup_rule(
        &self,
        rule_path: Path,
        current_scope: &Symbol,
    ) -> Option<(&GenericRule, &Symbol)> {
        // lookup scope by path; return `None` if scope doesn't exist
        self.scopes.get(&current_scope).and_then(|current_scope| {
            match (rule_path.scope(), rule_path.name()) {
                // if there is no included scope, get the rule from the current scope
                (None, rule_name) => current_scope
                    .rules
                    .get(&rule_name)
                    .map(|rule| (rule, &current_scope.name)),
                // if there is a scope name, check that the scope is included and get the rule from the included scope
                (Some(rule_scope), rule_name) => self
                    .get_included_scope(current_scope, rule_scope)
                    .and_then(|new_scope| {
                        new_scope
                            .rules
                            .get(&rule_name)
                            .map(|rule| (rule, &new_scope.name))
                    }),
            }
        })
    }

    /// Add `rule` to the rules for `scope`
    pub fn add_rule(&mut self, rule: Rule, scope: Symbol) -> Result<(), error::RuntimeError> {
        // lookup scope by path; panic if scope doesn't exist
        let scope = self
            .scopes
            .entry(scope.clone())
            .or_insert_with(|| Scope::new(scope));

        // determine if rule matches a rule template in the scope
        let rule_name = rule.name.clone();
        if let Some(rule_templates) = scope.rule_templates.get(&rule_name) {
            let mut has_template = false;
            let mut matched_template = false;
            for template in rule_templates {
                if rule.params.len() == template.params.len() {
                    // a rule has an applicable template if any template exists with the same name and arity
                    has_template = true;
                    // in order for a rule to have matched a template, the rule's parameters must exactly match
                    // the template's parameters
                    if rule.params == template.params {
                        matched_template = true;
                    }
                    // for (rule_param, template_param) in
                    //     rule.params.iter().zip(template.params.iter())
                    // {
                    //     match (&rule_param.specializer, &template_param.specializer) {
                    //         (Some(_rule_specializer), None) => (),
                    //         (Some(rule_specializer), Some(template_specializer)) => {
                    //             if rule_specializer != template_specializer {
                    //                 matched_template = false
                    //             }
                    //         }
                    //         _ => (),
                    //     }
                    // }
                }
            }
            // if the rule has at least one applicable template but did not match any, then it is not allowed
            if has_template && !matched_template {
                // TODO: warning or return code?
                return Err(error::RuntimeError::TypeError {
                    msg: "Rule not allowed in scope".to_owned(),
                    stack_trace: None,
                });
            }
        }

        let generic_rule = scope
            .rules
            .entry(rule_name.clone())
            .or_insert_with(|| GenericRule::new(rule_name, vec![]));
        Ok(generic_rule.add_rule(Arc::new(rule)))
    }

    /// Clear rules from KB, leaving constants in place.
    pub fn clear_rules(&mut self) {
        for (_, scope) in self.scopes.iter_mut() {
            scope.rules.clear()
        }
        self.sources = Sources::default();
        self.inline_queries.clear();
    }

    /// Add a rule template to the scope
    pub fn add_rule_template(&mut self, template: Rule, scope: Symbol) {
        let scope = self
            .scopes
            .entry(scope.clone())
            .or_insert_with(|| Scope::new(scope));

        // TODO: maybe check that rule body is empty?
        let name = template.name.clone();
        let _rule_templates = scope
            .rule_templates
            .entry(name.clone())
            .or_insert_with(|| vec![template]);
    }
}
