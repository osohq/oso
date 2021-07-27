use std::collections::HashMap;

pub use super::bindings::Bindings;
use super::counter::Counter;
use super::error::PolarResult;
use super::rules::*;
use super::sources::*;
use super::terms::*;

use std::sync::Arc;

/// A map of bindings: variable name → value. The VM uses a stack internally,
/// but can translate to and from this type.

// pub struct ScopeDefinition {
//     name: Symbol,

//     /// Scopes that you can call rules from this scope.
//     // included_names: HashSet<Path>,
//     // type definitions
// }

pub struct Scope {
    name: Symbol,
    // constants: Bindings,
    rule_templates: HashMap<Symbol, Vec<Rule>>,
    rules: HashMap<Symbol, GenericRule>,
}

impl Scope {
    pub fn new(name: Symbol) -> Self {
        Self {
            name: name,
            // constants: HashMap::new(),
            rule_templates: HashMap::new(),
            rules: HashMap::new(),
        }
    }
}

#[derive(Default)]
pub struct KnowledgeBase {
    pub constants: Bindings,
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
            constants: HashMap::new(),
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
        //     // All constants are defined on the default scope; if default scope doesn't exist, add it
        //     self.scopes
        //         .entry(sym!("default"))
        //         .or_insert(Scope::new(sym!("default").into()))
        //         .constants
        //         .insert(name, value);
        self.constants.insert(name, value);
    }

    /// Return true if a constant with the given name has been defined.
    pub fn is_constant(&self, symbol: &Symbol) -> bool {
        //     self.lookup_constant(Path::with_name(symbol.clone()), &sym!("default"))
        //         .is_some()
        self.constants.contains_key(symbol)
    }

    pub fn add_scope_definition(
        &mut self,
        name: Symbol,
        rule_templates: Vec<Rule>,
    ) -> PolarResult<()> {
        // Scope cannot already be defined.
        if self.scopes.contains_key(&name) {
            return Err(error::RuntimeError::FileLoading {
                msg: format!("Scope {} already defined.", name),
            }
            .into());
        }

        let scope = Scope::new(name.clone());
        self.scopes.insert(scope.name.clone(), scope);

        for template in rule_templates {
            // self.add_rule(template, name.clone())?;
            self.add_rule_template(template, &name)?;
        }

        Ok(())
    }

    pub fn add_scope_body(&mut self, name: Symbol, rules: Vec<Rule>) -> PolarResult<()> {
        if self.scopes.contains_key(&name) {
            for rule in rules {
                self.add_rule(rule, name.clone())?;
            }
        } else {
            return Err(error::RuntimeError::FileLoading {
                msg: format!(
                    "Scope {} must be defined with `def scope {}{{}}`.",
                    &name, &name
                ),
            }
            .into());
        }
        Ok(())
    }

    // pub fn lookup_constant(&self, const_path: Path, scope: &Symbol) -> Option<&Term> {
    //     // lookup scope by path; return `None` if scope doesn't exist
    //     self.scopes.get(&scope).and_then(|scope| {
    //         match (const_path.scope(), const_path.name()) {
    //             // if there is no included scope, get the constant from the current scope
    //             (None, const_name) => scope.constants.get(&const_name),
    //             // if there is an included scope, check that the scope is included and get the constant from the included scope
    //             (Some(included_scope), const_name) => self
    //                 .get_included_scope(scope, included_scope)
    //                 .and_then(|scope| scope.constants.get(&const_name)),
    //         }
    //     })
    // }

    /// Get `included` scope w.r.t `base`.
    fn get_included_scope(&self, base: &Scope, included: &Symbol) -> Option<&Scope> {
        if &base.name.0 == "default" {
            self.scopes.get(included)
        } else {
            None
        }
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
    pub fn get_rules(&self, scope: &Symbol) -> Option<&HashMap<Symbol, GenericRule>> {
        self.scopes.get(&scope).and_then(|scope| Some(&scope.rules))
    }

    /// Add `rule` to the rules for `scope`
    pub fn add_rule(&mut self, rule: Rule, scope_name: Symbol) -> Result<(), error::RuntimeError> {
        // lookup scope by path; panic if scope doesn't exist
        let scope = self
            .scopes
            .entry(scope_name.clone())
            .or_insert_with(|| Scope::new(scope_name.clone()));

        // determine if rule matches a rule template in the scope
        let rule_name = rule.name.clone();
        if let Some(rule_templates) = scope.rule_templates.get(&rule_name) {
            let mut arity_match = false;
            let mut matched_template = false;
            for template in rule_templates {
                if rule.params.len() == template.params.len() {
                    // a rule has an applicable template if any template exists with the same name and arity
                    arity_match = true;
                    // in order for a rule to have matched a template, the rule's parameters must exactly match
                    // the template's parameters
                    matched_template = KnowledgeBase::check_rule_compatibility(&rule, template);
                }
            }
            // if the rule has at least one applicable template but did not match any, then it is not allowed
            if arity_match && !matched_template {
                // TODO: warning or return code?
                return Err(error::RuntimeError::TypeError {
                    msg: format!(
                        "Rule {} with arity {} not allowed in scope because parameter types did not match rule template.",
                        rule_name,
                        rule.params.len()
                    ),
                    stack_trace: None,
                });
            } else if !rule_templates.is_empty() && !arity_match {
                return Err(error::RuntimeError::TypeError {
                    msg: format!("Rule {} with arity {} not allowed in scope because arity did not match rule template", rule_name, rule.params.len()),
                    stack_trace: None,
                });
            }
        } else if scope_name.0 != "default" {
            return Err(error::RuntimeError::TypeError {
                msg: format!(
                    "No rule template found for rule {} with arity {} in scope {}",
                    rule_name,
                    rule.params.len(),
                    scope_name
                ),
                stack_trace: None,
            });
        }

        let generic_rule = scope
            .rules
            .entry(rule_name.clone())
            .or_insert_with(|| GenericRule::new(rule_name, vec![]));
        Ok(generic_rule.add_rule(Arc::new(rule)))
    }

    pub fn check_rule_compatibility(rule: &Rule, template: &Rule) -> bool {
        for (rule_param, template_param) in rule.params.iter().zip(template.params.iter()) {
            let parameter_matches = match (
                template_param.parameter.value(),
                template_param.specializer.as_ref().map(Term::value),
                rule_param.parameter.value(),
                rule_param.specializer.as_ref().map(Term::value),
            ) {
                // Template (variable, specializer) then rule must have a variable and specializer that matches OR a value that matches the specializer.
                (
                    Value::Variable(_),
                    Some(Value::Pattern(Pattern::Instance(template_spec))),
                    Value::Variable(_),
                    Some(Value::Pattern(Pattern::Instance(rule_spec))),
                ) => {
                    // if tags match, all template fields must match those in rule fields, otherwise false
                    if template_spec.tag == rule_spec.tag {
                        let all_fields_match = template_spec
                            .fields
                            .fields
                            .iter()
                            .map(|(k, template_value)| {
                                rule_spec
                                    .fields
                                    .fields
                                    .get(k)
                                    .map(|rule_value| rule_value == template_value)
                                    .unwrap_or_else(|| false)
                            })
                            .all(|v| v);

                        all_fields_match
                    } else {
                        false
                    }
                }
                (Value::Variable(_), Some(_), Value::Variable(_), None) => false,
                (Value::Variable(_), Some(_template_spec), _rule_param, None) => {
                    // TODO: can't do this case right now
                    unimplemented!("value match spec not implemented");
                }
                // Template (variable, no specializer) then the rule can have anything, including any specializer
                (Value::Variable(_), None, _, _) => true,
                // Template (value, no specializer) the value must match exactly.
                (template_value, None, rule_value, None) => template_value == rule_value,
                _ => false,
            };

            if !parameter_matches {
                return false;
            }
        }

        true
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
    /// Precondition: scope exists with name `scope`.
    fn add_rule_template(&mut self, template: Rule, scope: &Symbol) -> PolarResult<()> {
        let scope = self.scopes.get_mut(scope).unwrap();

        // check that rule body is empty
        match template.body.value() {
            Value::Expression(Operation {
                operator: Operator::And,
                args,
            }) if args.is_empty() => (),
            _ => {
                return Err(error::RuntimeError::FileLoading {
                    msg: "Rule template cannot have a body.".to_owned(),
                }
                .into())
            }
        }
        let name = template.name.clone();
        // get rule templates
        scope
            .rule_templates
            .entry(name.clone())
            .and_modify(|rule_templates| rule_templates.push(template.clone()))
            .or_insert_with(|| vec![template]);

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_template_compatibility() {
        // Rules with variables allow any values.
        let template = rule!("f", [sym!("foo")]);
        let rule1 = rule!("f", [sym!("bar")]);
        let rule2 = rule!("f", [sym!("foo")]);
        let rule3 = rule!("f", [1]);
        let rule4 = rule!("f", [sym!("bar"); pattern!(instance!("Baz"))]);

        assert!(KnowledgeBase::check_rule_compatibility(&rule1, &template));
        assert!(KnowledgeBase::check_rule_compatibility(&rule2, &template));
        assert!(KnowledgeBase::check_rule_compatibility(&rule3, &template));
        assert!(KnowledgeBase::check_rule_compatibility(&rule4, &template));

        let template_with_value = rule!("g", [1]);
        let rule_g2 = rule!("g", [2]);
        assert!(KnowledgeBase::check_rule_compatibility(
            &template_with_value,
            &template_with_value
        ));
        assert!(!KnowledgeBase::check_rule_compatibility(
            &rule_g2,
            &template_with_value
        ));

        let template_spec = rule!("f", [sym!("foo"); pattern!(instance!("Bar")), sym!("baz"); pattern!(instance!("Baz"))]);
        let rule1 = rule!("f", [sym!("foo"); pattern!(instance!("Nope")), sym!("baz"); pattern!(instance!("Baz"))]);
        let rule2 = rule!("f", [sym!("foo"); pattern!(instance!("Bar")), sym!("baz"); pattern!(instance!("Nope"))]);

        assert!(KnowledgeBase::check_rule_compatibility(
            &template_spec,
            &template_spec
        ));
        assert!(!KnowledgeBase::check_rule_compatibility(
            &rule1,
            &template_spec
        ));
        assert!(!KnowledgeBase::check_rule_compatibility(
            &rule2,
            &template_spec
        ));
    }

    #[test]
    fn test_add_rule_template() {
        let mut kb = KnowledgeBase::new();

        let template = rule!("allow_role", [sym!("actor"); pattern!(instance!("User")), sym!("action"); pattern!(instance!("String")), sym!("resource"); pattern!(instance!("Repository"))]);

        // create scope first
        let scope_name = sym!("custom_scope");
        kb.scopes
            .insert(scope_name.clone(), Scope::new(scope_name.clone()));
        kb.add_rule_template(template, &scope_name).unwrap();
        // (actor: User, action: String, resource: Repository)")
        let rule = rule!("allow_role", [sym!("actor"); pattern!(instance!("User")), sym!("action"); pattern!(instance!("String")), sym!("resource"); pattern!(instance!("Repository"))]);
        assert!(kb.add_rule(rule, sym!("custom_scope")).is_ok());

        let bad_rule = rule!("allow_role", [sym!("actor"), sym!("action"); pattern!(instance!("String")), sym!("resource"); pattern!(instance!("Repository"))]);

        assert!(kb.add_rule(bad_rule, sym!("custom_scope")).is_err());
        let bad_rule = rule!("allow_role", [sym!("actor"); pattern!(instance!("EvilUser")), sym!("action"); pattern!(instance!("String")), sym!("resource"); pattern!(instance!("Repository"))]);
        assert!(kb.add_rule(bad_rule, sym!("custom_scope")).is_err());

        let template = rule!("allow_role", [sym!("actor"); pattern!(instance!("SuperUser")), sym!("action"); pattern!(instance!("String")), sym!("resource"); pattern!(instance!("Repository"))]);
        kb.add_rule_template(template, &scope_name).unwrap();
        let rule = rule!("allow_role", [sym!("actor"); pattern!(instance!("SuperUser")), sym!("action"); pattern!(instance!("String")), sym!("resource"); pattern!(instance!("Repository"))]);
        assert!(kb.add_rule(rule, sym!("custom_scope")).is_ok());
    }

    // TODO fields test.

    #[test]
    fn test_add_scope_def() {
        let mut kb = KnowledgeBase::new();
        let scope_name = sym!("custom_scope");
        let templates = vec![
            rule!("allow_role", [sym!("actor"); pattern!(instance!("User")), sym!("action"); pattern!(instance!("String")), sym!("resource"); pattern!(instance!("Repository"))]),
        ];

        // check adding a new scope works
        assert!(kb
            .add_scope_definition(scope_name.clone(), templates.clone())
            .is_ok());
        // check adding a scope that already exists fails
        assert!(kb.add_scope_definition(scope_name, templates).is_err());
        // check adding the default scope fails
        assert!(kb.add_scope_definition(sym!("default"), vec![]).is_err());
    }
}
