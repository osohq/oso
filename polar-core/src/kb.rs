use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::sync::Arc;

pub use super::bindings::Bindings;
use super::constants::Constants;
use super::counter::Counter;
use super::diagnostic::Diagnostic;
use super::error::{invalid_state, PolarError, PolarResult, RuntimeError, ValidationError};
use super::resource_block::{ResourceBlocks, ACTOR_UNION_NAME, RESOURCE_UNION_NAME};
use super::rules::*;
use super::terms::*;
use super::validations::check_undefined_rule_calls;

enum RuleParamMatch {
    True,
    False(String),
}

#[cfg(test)]
impl RuleParamMatch {
    fn is_true(&self) -> bool {
        matches!(self, RuleParamMatch::True)
    }
}

#[derive(Default)]
pub struct KnowledgeBase {
    /// A map of bindings: variable name → value. The VM uses a stack internally,
    /// but can translate to and from this type.
    constants: Constants,
    /// Map of class name -> MRO list where the MRO list is a list of class instance IDs
    pub mro: HashMap<Symbol, Vec<u64>>,

    /// Map from contents to filename for files loaded into the KB.
    loaded_content: HashMap<String, String>,

    rules: HashMap<Symbol, GenericRule>,
    rule_types: RuleTypes,
    /// For symbols returned from gensym.
    gensym_counter: Counter,
    /// For call IDs, instance IDs, symbols, etc.
    id_counter: Counter,
    pub inline_queries: Vec<Term>,

    /// Resource block bookkeeping.
    pub resource_blocks: ResourceBlocks,
}

impl KnowledgeBase {
    pub fn new() -> Self {
        Self::default()
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

    pub fn add_rule(&mut self, rule: Rule) {
        let generic_rule = self
            .rules
            .entry(rule.name.clone())
            .or_insert_with(|| GenericRule::new(rule.name.clone(), vec![]));
        generic_rule.add_rule(Arc::new(rule));
    }

    pub fn validate_rules(&self) -> Vec<Diagnostic> {
        // Prior to #1310 these validations were not order dependent due to the
        // use of static default rule types.
        // Now that rule types are dynamically generated based on policy
        // contents we validate types first to surface missing required rule
        // implementations which would otherwise raise opaque "call to undefined rule"
        // errors
        let mut diagnostics = vec![];

        if let Err(e) = self.validate_rule_types() {
            diagnostics.push(e.into());
        }

        diagnostics.append(&mut check_undefined_rule_calls(self));

        diagnostics
    }

    /// Validate that all rules loaded into the knowledge base are valid based on rule types.
    fn validate_rule_types(&self) -> PolarResult<()> {
        // For every rule, if there *is* a rule type, check that the rule matches the rule type.
        for (rule_name, generic_rule) in &self.rules {
            if let Some(types) = self.rule_types.get(rule_name) {
                // If a type with the same name exists, then the parameters must match for each rule
                for rule in generic_rule.rules.values() {
                    let mut msg = "Must match one of the following rule types:\n".to_owned();

                    let results = types
                        .iter()
                        .map(|rule_type| {
                            self.rule_params_match(rule.as_ref(), rule_type)
                                .map(|result| (result, rule_type))
                        })
                        .collect::<PolarResult<Vec<_>>>()?;
                    let found_match = results.iter().any(|(result, rule_type)| match result {
                        RuleParamMatch::True => true,
                        RuleParamMatch::False(message) => {
                            write!(
                                msg,
                                "\n{}\n\tFailed to match because: {}\n",
                                rule_type, message
                            )
                            .unwrap();
                            false
                        }
                    });
                    if !found_match {
                        let rule = Rule::clone(rule);
                        return Err(ValidationError::InvalidRule { rule, msg }.into());
                    }
                }
            }
        }

        // For every rule type that is *required*, see that there is at least one corresponding
        // implementation.
        for rule_type in self.rule_types.required_rule_types() {
            if let Some(GenericRule { rules, .. }) = self.rules.get(&rule_type.name) {
                let mut found_match = false;
                for rule in rules.values() {
                    found_match = self
                        .rule_params_match(rule.as_ref(), rule_type)
                        .map(|r| matches!(r, RuleParamMatch::True))?;
                    if found_match {
                        break;
                    }
                }
                if !found_match {
                    let rule_type = rule_type.clone();
                    return Err(ValidationError::MissingRequiredRule { rule_type }.into());
                }
            } else {
                let rule_type = rule_type.clone();
                return Err(ValidationError::MissingRequiredRule { rule_type }.into());
            }
        }

        Ok(())
    }

    /// Determine whether the fields of a rule parameter specializer match the fields of a type parameter specializer.
    /// Rule fields match if they are a superset of type fields and all field values are equal.
    // TODO: once field-level specializers are working this should be updated so
    // that it recursively checks all fields match, rather than checking for
    // equality
    fn param_fields_match(&self, type_fields: &Dictionary, rule_fields: &Dictionary) -> bool {
        return type_fields
            .fields
            .iter()
            .map(|(k, type_value)| {
                rule_fields
                    .fields
                    .get(k)
                    .map(|rule_value| rule_value == type_value)
                    .unwrap_or_else(|| false)
            })
            .all(|v| v);
    }

    /// Use MRO lists passed in from host library to determine if one `InstanceLiteral` pattern is
    /// a subclass of another `InstanceLiteral` pattern. This function is used for Rule Type
    /// validation.
    fn check_rule_instance_is_subclass_of_rule_type_instance(
        &self,
        rule_instance: &InstanceLiteral,
        rule_type_instance: &InstanceLiteral,
        index: usize,
    ) -> PolarResult<RuleParamMatch> {
        // Get the unique ID of the prototype instance pattern class.
        // TODO(gj): make actual term available here instead of constructing a fake test one.
        let term = self.get_registered_class(&term!(rule_type_instance.tag.clone()))?;
        if let Value::ExternalInstance(ExternalInstance { instance_id, .. }) = term.value() {
            if let Some(rule_mro) = self.mro.get(&rule_instance.tag) {
                if !rule_mro.contains(instance_id) {
                    Ok(RuleParamMatch::False(format!(
                        "Rule specializer {} on parameter {} must match rule type specializer {}",
                        rule_instance.tag, index, rule_type_instance.tag
                    )))
                } else if !self
                    .param_fields_match(&rule_type_instance.fields, &rule_instance.fields)
                {
                    Ok(RuleParamMatch::False(format!("Rule specializer {} on parameter {} did not match rule type specializer {} because the specializer fields did not match.", rule_instance, index, rule_type_instance)))
                } else {
                    Ok(RuleParamMatch::True)
                }
            } else {
                // If `rule_instance.tag` were registered as a class, it would have an MRO.
                Ok(RuleParamMatch::False(format!(
                    "Rule specializer {} on parameter {} is not registered as a class.",
                    rule_instance.tag, index
                )))
            }
        } else {
            Ok(RuleParamMatch::False(format!(
                "Rule type specializer {} on parameter {} should be a registered class, but instead it's registered as a constant with value: {}",
                rule_type_instance.tag, index, term
            )))
        }
    }

    /// Check that a rule parameter that has a pattern specializer matches a rule type parameter that has a pattern specializer.
    fn check_pattern_param(
        &self,
        index: usize,
        rule_pattern: &Pattern,
        rule_type_pattern: &Pattern,
    ) -> PolarResult<RuleParamMatch> {
        Ok(match (rule_type_pattern, rule_pattern) {
            (Pattern::Instance(rule_type_instance), Pattern::Instance(rule_instance)) => {
                // if tags match, all rule type fields must match those in rule fields, otherwise false
                if rule_type_instance.tag == rule_instance.tag {
                    if self.param_fields_match(
                        &rule_type_instance.fields,
                        &rule_instance.fields,
                    ) {
                        RuleParamMatch::True
                    } else {
                        RuleParamMatch::False(format!("Rule specializer {} on parameter {} did not match rule type specializer {} because the specializer fields did not match.", rule_instance, index, rule_type_instance))
                    }
                } else if self.is_union(&term!(sym!(&rule_type_instance.tag.0))) {
                    if self.is_union(&term!(sym!(&rule_instance.tag.0))) {
                        // If both specializers are the same union, check fields.
                        if rule_instance.tag == rule_type_instance.tag {
                            if self.param_fields_match(
                                &rule_type_instance.fields,
                                &rule_instance.fields,
                            ) {
                                return Ok(RuleParamMatch::True);
                            } else {
                                return Ok(RuleParamMatch::False(format!("Rule specializer {} on parameter {} did not match rule type specializer {} because the specializer fields did not match.", rule_instance, index, rule_type_instance)));
                            }
                        } else {
                            // TODO(gj): revisit when we have unions beyond Actor & Resource. Union
                            // A matches union B if union A is a member of union B.
                            return Ok(RuleParamMatch::False(format!("Rule specializer {} on parameter {} does not match rule type specializer {}", rule_instance.tag, index, rule_type_instance.tag)));
                        }
                    }

                    let members = self.get_union_members(&term!(sym!(&rule_type_instance.tag.0)));
                    // If the rule specializer is not a direct member of the union, we still need
                    // to check if it's a subclass of any member of the union.
                    if !members.contains(&term!(sym!(&rule_instance.tag.0))) {
                        let mut success = false;
                        for member in members {
                            // Turn `member` into an `InstanceLiteral` by copying fields from
                            // `rule_type_instance`.
                            let rule_type_instance = InstanceLiteral {
                                tag: member.as_symbol()?.clone(),
                                fields: rule_type_instance.fields.clone()
                            };
                            match self.check_rule_instance_is_subclass_of_rule_type_instance(rule_instance, &rule_type_instance, index) {
                                Ok(RuleParamMatch::True) if !success => success = true,
                                Err(e) => return Err(e),
                                _ => (),
                            }
                        }
                        if !success {
                            let mut err = format!("Rule specializer {} on parameter {} must be a member of rule type specializer {}", rule_instance.tag,index, rule_type_instance.tag);
                            if rule_type_instance.tag.0 == ACTOR_UNION_NAME {
                                write!(err, "

\tPerhaps you meant to add an actor block to the top of your policy, like this:

\t  actor {} {{}}", rule_instance.tag).unwrap();
                            } else if rule_type_instance.tag.0 == RESOURCE_UNION_NAME {
                                write!(err, "

\tPerhaps you meant to add a resource block to your policy, like this:

\t  resource {} {{ .. }}", rule_instance.tag).unwrap();

                            }

                            return Ok(RuleParamMatch::False(err));
                        }
                    }
                    if !self.param_fields_match(&rule_type_instance.fields, &rule_instance.fields) {
                        RuleParamMatch::False(format!("Rule specializer {} on parameter {} did not match rule type specializer {} because the specializer fields did not match.", rule_instance, index, rule_type_instance))
                    } else {
                        RuleParamMatch::True
                    }
                // If tags don't match, then rule specializer must be a subclass of rule type specializer
                } else {
                    self.check_rule_instance_is_subclass_of_rule_type_instance(rule_instance, rule_type_instance, index)?
                }
            }
            (Pattern::Dictionary(rule_type_fields), Pattern::Dictionary(rule_fields))
            | (Pattern::Dictionary(rule_type_fields), Pattern::Instance(InstanceLiteral { fields: rule_fields, .. })) => {
                if self.param_fields_match(rule_type_fields, rule_fields) {
                    RuleParamMatch::True
                } else {
                    RuleParamMatch::False(format!("Specializer mismatch on parameter {}. Rule specializer fields {:#?} do not match rule type specializer fields {:#?}.", index, rule_fields, rule_type_fields))
                }
            }
            (
                Pattern::Instance(InstanceLiteral {
                    tag,
                    fields: rule_type_fields,
                }),
                Pattern::Dictionary(rule_fields),
            ) if tag == &sym!("Dictionary") => {
                if self.param_fields_match(rule_type_fields, rule_fields) {
                    RuleParamMatch::True
                } else {
                    RuleParamMatch::False(format!("Specializer mismatch on parameter {}. Rule specializer fields {:#?} do not match rule type specializer fields {:#?}.", index, rule_fields, rule_type_fields))
                }
            }
            (_, _) => {
                RuleParamMatch::False(format!("Mismatch on parameter {}. Rule parameter {:#?} does not match rule type parameter {:#?}.", index, rule_type_pattern, rule_pattern))
            }
        })
    }

    /// Check that a rule parameter that is a value matches a rule type parameter that is a value
    fn check_value_param(
        &self,
        index: usize,
        rule_value: &Value,
        rule_type_value: &Value,
        rule_type: &Rule,
    ) -> PolarResult<RuleParamMatch> {
        Ok(match (rule_type_value, rule_value) {
            // List in rule head must be equal to or more specific than the list in the rule type head in order to match
            (Value::List(rule_type_list), Value::List(rule_list)) => {
                if has_rest_var(rule_type_list) {
                    return Err(ValidationError::InvalidRuleType {
                        rule_type: rule_type.clone(),
                        msg: "Rule types cannot contain *rest variables.".to_string(),
                    }
                    .into());
                }
                if rule_type_list.iter().all(|t| rule_list.contains(t)) {
                    RuleParamMatch::True
                } else {
                    RuleParamMatch::False(format!(
                        "Invalid parameter {}. Rule type expected list {:#?}, got list {:#?}.",
                        index, rule_type_list, rule_list
                    ))
                }
            }
            (Value::Dictionary(rule_type_fields), Value::Dictionary(rule_fields)) => {
                if self.param_fields_match(rule_type_fields, rule_fields) {
                    RuleParamMatch::True
                } else {
                    RuleParamMatch::False(format!("Invalid parameter {}. Rule type expected Dictionary with fields {:#?}, got Dictionary with fields {:#?}", index, rule_type_fields, rule_fields
                        ))
                }
            }
            (_, _) => {
                if rule_type_value == rule_value {
                    RuleParamMatch::True
                } else {
                    RuleParamMatch::False(format!(
                        "Invalid parameter {}. Rule value {} != rule type value {}",
                        index, rule_value, rule_type_value
                    ))
                }
            }
        })
    }
    /// Check a single rule parameter against a rule type parameter.
    fn check_param(
        &self,
        index: usize,
        rule_param: &Parameter,
        rule_type_param: &Parameter,
        rule_type: &Rule,
    ) -> PolarResult<RuleParamMatch> {
        Ok(
            match (
                rule_type_param.parameter.value(),
                rule_type_param.specializer.as_ref().map(Term::value),
                rule_param.parameter.value(),
                rule_param.specializer.as_ref().map(Term::value),
            ) {
                // Rule and rule type both have pattern specializers
                (
                    Value::Variable(_),
                    Some(Value::Pattern(rule_type_spec)),
                    Value::Variable(_),
                    Some(Value::Pattern(rule_spec)),
                ) => self.check_pattern_param(index, rule_spec, rule_type_spec)?,
                // RuleType has an instance pattern specializer but rule has no specializer
                (
                    Value::Variable(_),
                    Some(Value::Pattern(Pattern::Instance(InstanceLiteral { tag, .. }))),
                    Value::Variable(parameter),
                    None,
                ) => RuleParamMatch::False(format!(
                    "Parameter `{parameter}` expects a {tag} type constraint.

\t{parameter}: {tag}",
                    parameter = parameter,
                    tag = tag
                )),
                // RuleType has specializer but rule doesn't
                (Value::Variable(_), Some(rule_type_spec), Value::Variable(_), None) => {
                    RuleParamMatch::False(format!(
                        "Invalid rule parameter {}. Rule type expected {}",
                        index, rule_type_spec
                    ))
                }
                // Rule has value or value specializer, rule type has pattern specializer
                (
                    Value::Variable(_),
                    Some(Value::Pattern(rule_type_spec)),
                    Value::Variable(_),
                    Some(rule_value),
                )
                | (Value::Variable(_), Some(Value::Pattern(rule_type_spec)), rule_value, None) => {
                    match rule_type_spec {
                        // Rule type specializer is an instance pattern
                        Pattern::Instance(InstanceLiteral { .. }) => {
                            let rule_spec = match rule_value {
                                Value::String(_) => instance!(sym!("String")),
                                Value::Number(Numeric::Integer(_)) => instance!(sym!("Integer")),
                                Value::Number(Numeric::Float(_)) => instance!(sym!("Float")),
                                Value::Boolean(_) => instance!(sym!("Boolean")),
                                Value::List(_) => instance!(sym!("List")),
                                Value::Dictionary(rule_fields) => {
                                    instance!(sym!("Dictionary"), rule_fields.clone().fields)
                                }
                                _ => {
                                    // TODO(gj): what type of value could this be? Will this get
                                    // past the parser or is it unreachable? Prior to #1356 we
                                    // could hit this branch with a `Value::Variable` if the
                                    // specializer in the rule head was parenthesized.
                                    return invalid_state(format!(
                                        "Value variant {} cannot be a specializer",
                                        rule_value
                                    ));
                                }
                            };
                            self.check_pattern_param(
                                index,
                                &Pattern::Instance(rule_spec),
                                rule_type_spec,
                            )?
                        }
                        // Rule type specializer is a dictionary pattern
                        Pattern::Dictionary(rule_type_fields) => {
                            if let Value::Dictionary(rule_fields) = rule_value {
                                if self.param_fields_match(rule_type_fields, rule_fields) {
                                    RuleParamMatch::True
                                } else {
                                    RuleParamMatch::False(format!("Invalid parameter {}. Rule type expected Dictionary with fields {}, got dictionary with fields {}.", index, rule_type_fields, rule_fields))
                                }
                            } else {
                                RuleParamMatch::False(format!(
                                    "Invalid parameter {}. Rule type expected Dictionary, got {}.",
                                    index, rule_value
                                ))
                            }
                        }
                    }
                }

                // Rule type has no specializer
                (Value::Variable(_), None, _, _) => RuleParamMatch::True,
                // Rule has value or value specializer, rule type has value specializer |
                // rule has value, rule type has value
                (
                    Value::Variable(_),
                    Some(rule_type_value),
                    Value::Variable(_),
                    Some(rule_value),
                )
                | (Value::Variable(_), Some(rule_type_value), rule_value, None)
                | (rule_type_value, None, rule_value, None) => {
                    self.check_value_param(index, rule_value, rule_type_value, rule_type)?
                }
                _ => RuleParamMatch::False(format!(
                    "Invalid parameter {}. Rule parameter {} does not match rule type parameter {}",
                    index, rule_param, rule_type_param
                )),
            },
        )
    }

    /// Determine whether a `rule` matches a `rule_type` based on its parameters.
    fn rule_params_match(&self, rule: &Rule, rule_type: &Rule) -> PolarResult<RuleParamMatch> {
        if rule.params.len() != rule_type.params.len() {
            return Ok(RuleParamMatch::False(format!(
                "Different number of parameters. Rule has {} parameter(s) but rule type has {}.",
                rule.params.len(),
                rule_type.params.len()
            )));
        }
        let mut failure_message = "".to_owned();
        rule.params
            .iter()
            .zip(rule_type.params.iter())
            .enumerate()
            .map(|(i, (rule_param, rule_type_param))| {
                self.check_param(i + 1, rule_param, rule_type_param, rule_type)
            })
            .collect::<PolarResult<Vec<RuleParamMatch>>>()
            .map(|results| {
                // TODO(gj): all() is short-circuiting -- do we want to gather up *all* failure
                // messages instead of just the first one?
                results.iter().all(|r| {
                    if let RuleParamMatch::False(msg) = r {
                        failure_message = msg.to_owned();
                        false
                    } else {
                        true
                    }
                })
            })
            .map(|matched| {
                if matched {
                    RuleParamMatch::True
                } else {
                    RuleParamMatch::False(failure_message)
                }
            })
    }

    pub fn get_rules(&self) -> &HashMap<Symbol, GenericRule> {
        &self.rules
    }

    #[cfg(test)]
    pub fn get_rule_types(&self, name: &Symbol) -> Option<&Vec<Rule>> {
        self.rule_types.get(name)
    }

    pub fn get_generic_rule(&self, name: &Symbol) -> Option<&GenericRule> {
        self.rules.get(name)
    }

    pub fn add_rule_type(&mut self, rule_type: Rule) {
        self.rule_types.add(rule_type);
    }

    /// Define a constant variable.
    ///
    /// Error on attempts to register the "union" types (Actor & Resource) since those types have
    /// special meaning in policies that use resource blocks.
    pub fn register_constant(&mut self, name: Symbol, value: Term) -> PolarResult<()> {
        if name.0 == ACTOR_UNION_NAME || name.0 == RESOURCE_UNION_NAME {
            return Err(RuntimeError::InvalidRegistration {
                msg: format!("'{}' is a built-in specializer.", name),
                sym: name,
            }
            .into());
        }

        if let Value::ExternalInstance(ExternalInstance {
            class_id,
            instance_id,
            ..
        }) = *value.value()
        {
            if class_id.map_or(false, |id| id == instance_id) {
                // ExternalInstance values with matching class_id & instance_id represent *classes*
                // whose class_id we want to index for later type checking & MRO resolution
                self.constants.insert_class(name, value, instance_id)
            } else {
                // ExternalInstance values with differing `class_id` and
                // `instance_id` represent *instances* of classes whose class_id
                // should not be registered
                self.constants.insert(name, value)
            }
        } else {
            self.constants.insert(name, value)
        }
        Ok(())
    }

    /// Return true if a constant with the given name has been defined.
    pub fn is_constant(&self, name: &Symbol) -> bool {
        self.constants.contains_key(name)
    }

    /// Getter for `constants` map without exposing it for mutation.
    pub fn get_registered_constants(&self) -> &Bindings {
        &self.constants.symbol_to_term
    }

    pub(crate) fn get_symbol_for_class_id(&self, id: &u64) -> Option<&Symbol> {
        self.constants.get_symbol_for_class_id(id)
    }

    pub(crate) fn get_class_id_for_symbol(&self, symbol: &Symbol) -> Option<&u64> {
        self.constants.get_class_id_for_symbol(symbol)
    }

    // TODO(gj): currently no way to distinguish classes from other registered constants in the
    // core, so it's up to callers to ensure this is only called with terms we expect to be
    // registered as a _class_.
    pub fn get_registered_class(&self, class: &Term) -> PolarResult<&Term> {
        self.constants.get(class.as_symbol()?).ok_or_else(|| {
            ValidationError::UnregisteredClass {
                term: class.clone(),
            }
            .into()
        })
    }

    /// Add the Method Resolution Order (MRO) list for a registered class.
    /// The `mro` argument is a list of the `instance_id` associated with a registered class.
    pub fn add_mro(&mut self, name: Symbol, mro: Vec<u64>) -> PolarResult<()> {
        // Confirm name is a registered class
        if !self.is_constant(&name) {
            return invalid_state(format!("Cannot add MRO for unregistered class {}", name));
        }
        self.mro.insert(name, mro);
        Ok(())
    }

    pub fn clear_rules(&mut self) {
        self.rules.clear();
        self.rule_types.reset();
        self.inline_queries.clear();
        self.loaded_content.clear();
        self.resource_blocks.clear();
    }

    // TODO(gj): Remove this fn & `FileLoading` error variant. These checks don't spark joy.
    pub(crate) fn add_source(&mut self, filename: &str, contents: &str) -> PolarResult<()> {
        let seen_filename = self.loaded_content.values().any(|name| name == filename);
        match self.loaded_content.insert(contents.into(), filename.into()) {
            Some(other_file) if other_file == filename => {
                Err(format!("File {} has already been loaded.", filename))
            }
            Some(other_file) => Err(format!(
                "A file with the same contents as {} named {} has already been loaded.",
                filename, other_file
            )),
            _ if seen_filename => Err(format!(
                "A file with the name {}, but different contents has already been loaded.",
                filename
            )),
            _ => Ok(()),
        }
        .map_err(|msg| {
            ValidationError::FileLoading {
                filename: filename.into(),
                contents: contents.into(),
                msg,
            }
            .into()
        })
    }

    /// Check that all relations declared across all resource blocks have been registered as
    /// constants.
    fn check_that_resource_block_relations_are_registered(&self) -> Vec<PolarError> {
        self.resource_blocks
            .relation_tuples()
            .into_iter()
            .filter_map(|(relation_type, _, _)| self.get_registered_class(relation_type).err())
            .collect()
    }

    pub fn rewrite_shorthand_rules(&mut self) -> Vec<PolarError> {
        let mut errors = vec![];

        errors.append(&mut self.check_that_resource_block_relations_are_registered());

        let mut rules = vec![];
        for (resource_name, shorthand_rules) in &self.resource_blocks.shorthand_rules {
            for shorthand_rule in shorthand_rules {
                match shorthand_rule.as_rule(resource_name, &self.resource_blocks) {
                    Ok(rule) => rules.push(rule),
                    Err(error) => errors.push(error),
                }
            }
        }

        // Add the rewritten rules to the KB.
        for rule in rules {
            self.add_rule(rule);
        }

        errors
    }

    pub fn create_resource_specific_rule_types(&mut self) -> PolarResult<()> {
        let mut rule_types_to_create = HashMap::new();

        // TODO @patrickod refactor RuleTypes & split out
        // RequiredRuleType struct to record the related
        // shorthand rule and relation terms.

        // Iterate through all resource block declarations and create
        // non-required rule types for each relation declaration we observe.
        //
        // We create non-required rule types to gracefully account for the case
        // where users have declared relations ahead of time that are used in
        // rule or resource definitions.
        for (subject, name, object) in self.resource_blocks.relation_tuples() {
            rule_types_to_create.insert((subject, name, object), false);
        }

        // Iterate through resource block shorthand rules and create *required*
        // rule types for each relation which is traversed in the rules.
        for (object, shorthand_rules) in &self.resource_blocks.shorthand_rules {
            for shorthand_rule in shorthand_rules {
                // We create rule types from shorthand rules in the following scenarios...
                match &shorthand_rule.body {
                    // 1. When the the third "relation" term points to a related Resource. E.g.,
                    //    `"admin" if "admin" on "parent";` where `relations = { parent: Org };`.
                    (implier, Some((_, relation))) => {
                        // First, create required rule type for relationship between `object` and
                        // `subject`:
                        //
                        // resource Repo {
                        //   roles = ["writer"];
                        //   relations = { parent_org: Org };
                        //
                        //   "writer" if "admin" on "parent_org";
                        // }
                        //
                        // (required) type has_relation(org: Org, "parent_org", repo: Repo);
                        //
                        // resource Org {
                        //   roles = ["admin"];
                        // }
                        if let Ok(subject) = self
                            .resource_blocks
                            .get_relation_type_in_resource_block(relation, object)
                        {
                            rule_types_to_create.insert((subject, relation, object), true);

                            // Then, if the "implier" term is declared as a relation on `subject`
                            // (as opposed to a permission or role), create required rule type for
                            // relationship between `related_subject` and `subject`:
                            //
                            // resource Repo {
                            //   roles = ["writer"];
                            //   relations = { parent_org: Org };
                            //
                            //   "writer" if "owner" on "parent_org";
                            // }
                            //
                            // (required) type has_relation(org: Org, "parent_org", issue: Issue);
                            //
                            // resource Org {
                            //   relations = { owner: User };
                            // }
                            //
                            // (required) type has_relation(user: User, "owner", org: Org);
                            if let Ok(related_subject) = self
                                .resource_blocks
                                .get_relation_type_in_resource_block(implier, subject)
                            {
                                rule_types_to_create
                                    .insert((related_subject, implier, subject), true);
                            }
                        }
                    }

                    // 2. When the second "implier" term points to a related Actor. E.g., `"admin"
                    //    if "owner";` where `relations = { owner: User };`. Technically, "implier"
                    //    could be a related Resource, but that doesn't make much semantic sense.
                    //    Related resources should be traversed via `"on"` clauses, which are
                    //    captured in the above match arm.
                    (implier, None) => {
                        if let Ok(subject) = self
                            .resource_blocks
                            .get_relation_type_in_resource_block(implier, object)
                        {
                            rule_types_to_create.insert((subject, implier, object), true);
                        }
                    }
                }
            }
        }

        let mut rule_types = rule_types_to_create.into_iter().map(|((subject, relation, object), required)| {
            let subject_specializer = pattern!(instance!(&subject.as_symbol()?.0));
            let relation_name = relation.as_string()?;
            let object_specializer = pattern!(instance!(&object.as_symbol()?.0));

            let name = sym!("has_relation");
            let mut params = args!("subject"; subject_specializer, relation_name, "object"; object_specializer);
            params.reverse();
            let body = term!(op!(And));
            // Copy SourceInfo from implier or relation in shorthand rule.
            let source_info = relation.source_info().clone();
            Ok(Rule { name, params, body, source_info, required })
        }).collect::<PolarResult<Vec<_>>>()?;

        // If there are any Relation::Role declarations in *any* of our resource
        // blocks then we want to add the `has_role` rule type.
        if self.resource_blocks.has_roles() {
            rule_types.push(
                // TODO(gj): "Internal" SourceInfo variant.
                // TODO(gj): Figure out if it's worth setting SourceInfo::Parser context for this
                // `has_role` rule type we create. Best we could probably do at the moment is fetch
                // a random role from self.resource_blocks.declarations and borrow its context.
                rule!("has_role", ["actor"; instance!(ACTOR_UNION_NAME), "role"; instance!("String"), "resource"; instance!(RESOURCE_UNION_NAME)], true)
            );
        }

        for rule_type in rule_types {
            self.add_rule_type(rule_type.clone());
        }
        Ok(())
    }

    pub fn is_union(&self, maybe_union: &Term) -> bool {
        (maybe_union.is_actor_union()) || (maybe_union.is_resource_union())
    }

    pub fn get_union_members(&self, union: &Term) -> &HashSet<Term> {
        if union.is_actor_union() {
            &self.resource_blocks.actors
        } else if union.is_resource_union() {
            &self.resource_blocks.resources
        } else {
            unreachable!()
        }
    }

    pub fn has_rules(&self) -> bool {
        !self.rules.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::error::ValidationError::{FileLoading, InvalidRule};

    #[test]
    fn test_add_source_file_validation() {
        fn expect_error(kb: &mut KnowledgeBase, name: &str, contents: &str, expected: &str) {
            let err = kb.add_source(name, contents).unwrap_err();
            let msg = match err.unwrap_validation() {
                FileLoading { msg, .. } => msg,
                e => panic!("Unexpected error: {}", e),
            };
            assert_eq!(msg, expected);
        }

        let mut kb = KnowledgeBase::new();
        let contents1 = "f();";
        let contents2 = "g();";
        let filename1 = "f";
        let filename2 = "g";

        // Load source1.
        kb.add_source(filename1, contents1).unwrap();

        // Cannot load source1 a second time.
        let expected = format!("File {} has already been loaded.", filename1);
        expect_error(&mut kb, filename1, contents1, &expected);

        // Cannot load source2 with the same name as source1 but different contents.
        let expected = format!(
            "A file with the name {}, but different contents has already been loaded.",
            filename1
        );
        expect_error(&mut kb, filename1, contents2, &expected);

        // Cannot load source3 with the same contents as source1 but a different name.
        let expected = format!(
            "A file with the same contents as {} named {} has already been loaded.",
            filename2, filename1
        );
        expect_error(&mut kb, filename2, contents1, &expected);
    }

    #[test]
    fn test_rule_params_match() {
        let mut kb = KnowledgeBase::new();

        let mut constant = |name: &str, instance_id: u64| {
            kb.register_constant(
                sym!(name),
                term!(Value::ExternalInstance(ExternalInstance {
                    instance_id,
                    constructor: None,
                    repr: None,
                    class_repr: None,
                    class_id: None,
                })),
            )
            .unwrap();
        };

        constant("Fruit", 1);
        constant("Citrus", 2);
        constant("Orange", 3);
        // NOTE: Foo doesn't need an MRO b/c it only appears as a rule type specializer; not a rule
        // specializer.
        constant("Foo", 4);

        // NOTE: this is only required for these tests b/c we're bypassing the normal load process,
        // where MROs are registered via FFI calls in the host language libraries.
        // process.
        constant("Integer", 5);
        constant("Float", 6);
        constant("String", 7);
        constant("Boolean", 8);
        constant("List", 9);
        constant("Dictionary", 10);

        kb.add_mro(sym!("Fruit"), vec![1]).unwrap();
        // Citrus is a subclass of Fruit
        kb.add_mro(sym!("Citrus"), vec![2, 1]).unwrap();
        // Orange is a subclass of Citrus
        kb.add_mro(sym!("Orange"), vec![3, 2, 1]).unwrap();

        kb.add_mro(sym!("Integer"), vec![]).unwrap();
        kb.add_mro(sym!("Float"), vec![]).unwrap();
        kb.add_mro(sym!("String"), vec![]).unwrap();
        kb.add_mro(sym!("Boolean"), vec![]).unwrap();
        kb.add_mro(sym!("List"), vec![]).unwrap();
        kb.add_mro(sym!("Dictionary"), vec![]).unwrap();

        // BOTH PATTERN SPEC
        // rule: f(x: Foo), rule_type: f(x: Foo) => PASS
        assert!(kb
            .rule_params_match(
                &rule!("f", ["x"; instance!(sym!("Fruit"))]),
                &rule!("f", ["x"; instance!(sym!("Fruit"))])
            )
            .unwrap()
            .is_true());

        // rule: f(x: Foo), rule_type: f(x: Bar) => FAIL if Foo is not subclass of Bar
        assert!(!kb
            .rule_params_match(
                &rule!("f", ["x"; instance!(sym!("Fruit"))]),
                &rule!("f", ["x"; instance!(sym!("Citrus"))])
            )
            .unwrap()
            .is_true());

        // rule: f(x: Foo), rule_type: f(x: Bar) => PASS if Foo is subclass of Bar
        assert!(kb
            .rule_params_match(
                &rule!("f", ["x"; instance!(sym!("Citrus"))]),
                &rule!("f", ["x"; instance!(sym!("Fruit"))])
            )
            .unwrap()
            .is_true());
        assert!(kb
            .rule_params_match(
                &rule!("f", ["x"; instance!(sym!("Orange"))]),
                &rule!("f", ["x"; instance!(sym!("Fruit"))])
            )
            .unwrap()
            .is_true());

        // rule: f(x: Foo), rule_type: f(x: {id: 1}) => FAIL
        assert!(!kb
            .rule_params_match(
                &rule!("f", ["x"; instance!(sym!("Foo"))]),
                &rule!("f", ["x"; btreemap! {sym!("id") => term!(1)}])
            )
            .unwrap()
            .is_true());
        // rule: f(x: Foo{id: 1}), rule_type: f(x: {id: 1}) => PASS
        assert!(kb
            .rule_params_match(
                &rule!(
                    "f",
                    ["x"; instance!(sym!("Foo"), btreemap! {sym!("id") => term!(1)})]
                ),
                &rule!("f", ["x"; btreemap! {sym!("id") => term!(1)}])
            )
            .unwrap()
            .is_true());
        // rule: f(x: {id: 1}), rule_type: f(x: Foo{id: 1}) => FAIL
        assert!(!kb
            .rule_params_match(
                &rule!("f", ["x"; btreemap! {sym!("id") => term!(1)}]),
                &rule!(
                    "f",
                    ["x"; instance!(sym!("Foo"), btreemap! {sym!("id") => term!(1)})]
                )
            )
            .unwrap()
            .is_true());
        // rule: f(x: {id: 1}), rule_type: f(x: {id: 1}) => PASS
        assert!(kb
            .rule_params_match(
                &rule!("f", ["x"; btreemap! {sym!("id") => term!(1)}]),
                &rule!("f", ["x"; btreemap! {sym!("id") => term!(1)}])
            )
            .unwrap()
            .is_true());

        // RULE VALUE SPEC, TEMPLATE PATTERN SPEC
        // rule: f(x: 6), rule_type: f(x: Integer) => PASS
        assert!(kb
            .rule_params_match(
                &rule!("f", ["x"; value!(6)]),
                &rule!("f", ["x"; instance!(sym!("Integer"))])
            )
            .unwrap()
            .is_true());

        // rule: f(x: 6), rule_type: f(x: Foo) => FAIL
        assert!(!kb
            .rule_params_match(
                &rule!("f", ["x"; value!(6)]),
                &rule!("f", ["x"; instance!(sym!("Foo"))])
            )
            .unwrap()
            .is_true());
        // rule: f(x: "string"), rule_type: f(x: Integer) => FAIL
        assert!(!kb
            .rule_params_match(
                &rule!("f", ["x"; value!("string")]),
                &rule!("f", ["x"; instance!(sym!("Integer"))])
            )
            .unwrap()
            .is_true());
        // rule: f(x: 6.0), rule_type: f(x: Float) => PASS
        assert!(kb
            .rule_params_match(
                &rule!("f", ["x"; value!(6.0)]),
                &rule!("f", ["x"; instance!(sym!("Float"))])
            )
            .unwrap()
            .is_true());
        // rule: f(x: 6.0), rule_type: f(x: Foo) => FAIL
        assert!(!kb
            .rule_params_match(
                &rule!("f", ["x"; value!(6.0)]),
                &rule!("f", ["x"; instance!(sym!("Foo"))])
            )
            .unwrap()
            .is_true());
        // rule: f(x: 6), rule_type: f(x: Float) => FAIL
        assert!(!kb
            .rule_params_match(
                &rule!("f", ["x"; value!(6)]),
                &rule!("f", ["x"; instance!(sym!("Float"))])
            )
            .unwrap()
            .is_true());
        // rule: f(x: "hi"), rule_type: f(x: String) => PASS
        assert!(kb
            .rule_params_match(
                &rule!("f", ["x"; value!("hi")]),
                &rule!("f", ["x"; instance!(sym!("String"))])
            )
            .unwrap()
            .is_true());
        // rule: f(x: "hi"), rule_type: f(x: Foo) => FAIL
        assert!(!kb
            .rule_params_match(
                &rule!("f", ["x"; value!("hi")]),
                &rule!("f", ["x"; instance!(sym!("Foo"))])
            )
            .unwrap()
            .is_true());
        // rule: f(x: 6), rule_type: f(x: String) => FAIL
        assert!(!kb
            .rule_params_match(
                &rule!("f", ["x"; value!(6)]),
                &rule!("f", ["x"; instance!(sym!("String"))])
            )
            .unwrap()
            .is_true());
        // Ensure primitive types cannot have fields
        // rule: f(x: "hello"), rule_type: f(x: String{id: 1}) => FAIL
        assert!(!kb
            .rule_params_match(
                &rule!("f", ["x"; value!("hello")]),
                &rule!(
                    "f",
                    ["x"; instance!(sym!("String"), btreemap! {sym!("id") => term!(1)})]
                )
            )
            .unwrap()
            .is_true());
        // rule: f(x: true), rule_type: f(x: Boolean) => PASS
        assert!(kb
            .rule_params_match(
                &rule!("f", ["x"; value!(true)]),
                &rule!("f", ["x"; instance!(sym!("Boolean"))])
            )
            .unwrap()
            .is_true());
        // rule: f(x: true), rule_type: f(x: Foo) => FAIL
        assert!(!kb
            .rule_params_match(
                &rule!("f", ["x"; value!(true)]),
                &rule!("f", ["x"; instance!(sym!("Foo"))])
            )
            .unwrap()
            .is_true());
        // rule: f(x: 6), rule_type: f(x: Boolean) => FAIL
        assert!(!kb
            .rule_params_match(
                &rule!("f", ["x"; value!(6)]),
                &rule!("f", ["x"; instance!(sym!("Boolean"))])
            )
            .unwrap()
            .is_true());
        // rule: f(x: [1, 2]), rule_type: f(x: List) => PASS
        assert!(kb
            .rule_params_match(
                &rule!("f", ["x"; value!([1, 2])]),
                &rule!("f", ["x"; instance!(sym!("List"))])
            )
            .unwrap()
            .is_true());
        // rule: f(x: [1, 2]), rule_type: f(x: Foo) => FAIL
        assert!(!kb
            .rule_params_match(
                &rule!("f", ["x"; value!([1, 2])]),
                &rule!("f", ["x"; instance!(sym!("Foo"))])
            )
            .unwrap()
            .is_true());
        // rule: f(x: 6), rule_type: f(x: List) => FAIL
        assert!(!kb
            .rule_params_match(
                &rule!("f", ["x"; value!(6)]),
                &rule!("f", ["x"; instance!(sym!("List"))])
            )
            .unwrap()
            .is_true());
        // rule: f(x: {id: 1}), rule_type: f(x: Dictionary) => PASS
        assert!(kb
            .rule_params_match(
                &rule!("f", ["x"; btreemap! {sym!("id") => term!(1)}]),
                &rule!("f", ["x"; instance!(sym!("Dictionary"))])
            )
            .unwrap()
            .is_true());
        // rule: f(x: {id: 1}), rule_type: f(x: Foo) => FAIL
        assert!(!kb
            .rule_params_match(
                &rule!("f", ["x"; btreemap! {sym!("id") => term!(1)}]),
                &rule!("f", ["x"; instance!(sym!("Foo"))])
            )
            .unwrap()
            .is_true());
        // rule: f({id: 1}), rule_type: f(x: Foo) => FAIL
        assert!(!kb
            .rule_params_match(
                &rule!("f", [btreemap! {sym!("id") => term!(1)}]),
                &rule!("f", ["x"; instance!(sym!("Foo"))])
            )
            .unwrap()
            .is_true());
        // rule: f(x: 6), rule_type: f(x: Dictionary) => FAIL
        assert!(!kb
            .rule_params_match(
                &rule!("f", ["x"; value!(6)]),
                &rule!("f", ["x"; instance!(sym!("Dictionary"))])
            )
            .unwrap()
            .is_true());
        // rule: f(x: {id: 1}), rule_type: f(x: Dictionary{id: 1}) => PASS
        assert!(kb
            .rule_params_match(
                &rule!("f", ["x"; btreemap! {sym!("id") => term!(1)}]),
                &rule!(
                    "f",
                    ["x"; instance!(sym!("Dictionary"), btreemap! {sym!("id") => term!(1)})]
                )
            )
            .unwrap()
            .is_true());

        // RULE PATTERN SPEC, TEMPLATE VALUE SPEC
        // always => FAIL
        assert!(!kb
            .rule_params_match(
                &rule!("f", ["x"; btreemap!(sym!("1") => term!(1))]),
                &rule!("f", ["x"; value!(1)])
            )
            .unwrap()
            .is_true());

        // BOTH VALUE SPEC
        // Integer, String, Boolean: must be equal
        // rule: f(x: 1), rule_type: f(x: 1) => PASS
        assert!(kb
            .rule_params_match(&rule!("f", ["x"; value!(1)]), &rule!("f", ["x"; value!(1)]))
            .unwrap()
            .is_true());
        // rule: f(x: 1), rule_type: f(x: 2) => FAIL
        assert!(!kb
            .rule_params_match(&rule!("f", ["x"; value!(1)]), &rule!("f", ["x"; value!(2)]))
            .unwrap()
            .is_true());
        // rule: f(x: 1.0), rule_type: f(x: 1.0) => PASS
        assert!(kb
            .rule_params_match(
                &rule!("f", ["x"; value!(1.0)]),
                &rule!("f", ["x"; value!(1.0)])
            )
            .unwrap()
            .is_true());
        // rule: f(x: 1.0), rule_type: f(x: 2.0) => FAIL
        assert!(!kb
            .rule_params_match(
                &rule!("f", ["x"; value!(1.0)]),
                &rule!("f", ["x"; value!(2.0)])
            )
            .unwrap()
            .is_true());
        // rule: f(x: "hi"), rule_type: f(x: "hi") => PASS
        assert!(kb
            .rule_params_match(
                &rule!("f", ["x"; value!("hi")]),
                &rule!("f", ["x"; value!("hi")])
            )
            .unwrap()
            .is_true());
        // rule: f(x: "hi"), rule_type: f(x: "hello") => FAIL
        assert!(!kb
            .rule_params_match(
                &rule!("f", ["x"; value!("hi")]),
                &rule!("f", ["x"; value!("hello")])
            )
            .unwrap()
            .is_true());
        // rule: f(x: true), rule_type: f(x: true) => PASS
        assert!(kb
            .rule_params_match(
                &rule!("f", ["x"; value!(true)]),
                &rule!("f", ["x"; value!(true)])
            )
            .unwrap()
            .is_true());
        // rule: f(x: true), rule_type: f(x: false) => PASS
        assert!(!kb
            .rule_params_match(
                &rule!("f", ["x"; value!(true)]),
                &rule!("f", ["x"; value!(false)])
            )
            .unwrap()
            .is_true());
        // List: rule must be more specific than (superset of) rule_type
        // rule: f(x: [1,2,3]), rule_type: f(x: [1,2]) => PASS
        // TODO: I'm not sure this logic actually makes sense--it feels like
        // they should have to be an exact match
        assert!(kb
            .rule_params_match(
                &rule!("f", ["x"; value!([1, 2, 3])]),
                &rule!("f", ["x"; value!([1, 2])])
            )
            .unwrap()
            .is_true());
        // rule: f(x: [1,2]), rule_type: f(x: [1,2,3]) => FAIL
        assert!(!kb
            .rule_params_match(
                &rule!("f", ["x"; value!([1, 2])]),
                &rule!("f", ["x"; value!([1, 2, 3])])
            )
            .unwrap()
            .is_true());
        // test with *rest vars
        // rule: f(x: [1, 2, 3]), rule_type: f(x: [1, 2, *rest]) => PASS
        assert!(kb
            .rule_params_match(
                &rule!("f", ["x"; value!([1, 2])]),
                &rule!(
                    "f",
                    ["x"; value!([1, 2, Value::RestVariable(sym!("*_rest"))])]
                )
            )
            .is_err());
        // Dict: rule must be more specific than (superset of) rule_type
        // rule: f(x: {"id": 1, "name": "Dave"}), rule_type: f(x: {"id": 1}) => PASS
        assert!(kb
            .rule_params_match(
                &rule!(
                    "f",
                    ["x"; btreemap! {sym!("id") => term!(1), sym!("name") => term!(sym!("Dave"))}]
                ),
                &rule!("f", ["x"; btreemap! {sym!("id") => term!(1)}]),
            )
            .unwrap()
            .is_true());
        // rule: f(x: {"id": 1}), rule_type: f(x: {"id": 1, "name": "Dave"}) => FAIL
        assert!(!kb
            .rule_params_match(
                &rule!("f", ["x"; btreemap! {sym!("id") => term!(1)}]),
                &rule!(
                    "f",
                    ["x"; btreemap! {sym!("id") => term!(1), sym!("name") => term!(sym!("Dave"))}]
                )
            )
            .unwrap()
            .is_true());

        // RULE None SPEC TEMPLATE Some SPEC
        // always => FAIL
        assert!(!kb
            .rule_params_match(
                &rule!("f", [sym!("x")]),
                &rule!("f", ["x"; instance!(sym!("Foo"))])
            )
            .unwrap()
            .is_true());

        // RULE Some SPEC TEMPLATE None SPEC
        // always => PASS
        assert!(kb
            .rule_params_match(
                &rule!("f", ["x"; instance!(sym!("Foo"))]),
                &rule!("f", [sym!("x")]),
            )
            .unwrap()
            .is_true());
    }

    #[test]
    fn test_validate_rules() {
        let mut kb = KnowledgeBase::new();
        kb.register_constant(
            sym!("Fruit"),
            term!(Value::ExternalInstance(ExternalInstance {
                instance_id: 1,
                constructor: None,
                repr: None,
                class_repr: None,
                class_id: None,
            })),
        )
        .unwrap();
        kb.register_constant(
            sym!("Citrus"),
            term!(Value::ExternalInstance(ExternalInstance {
                instance_id: 2,
                constructor: None,
                repr: None,
                class_repr: None,
                class_id: None,
            })),
        )
        .unwrap();
        kb.register_constant(
            sym!("Orange"),
            term!(Value::ExternalInstance(ExternalInstance {
                instance_id: 3,
                constructor: None,
                repr: None,
                class_repr: None,
                class_id: None,
            })),
        )
        .unwrap();
        kb.add_mro(sym!("Fruit"), vec![1]).unwrap();
        // Citrus is a subclass of Fruit
        kb.add_mro(sym!("Citrus"), vec![2, 1]).unwrap();
        // Orange is a subclass of Citrus
        kb.add_mro(sym!("Orange"), vec![3, 2, 1]).unwrap();

        // Rule type applies if it has the same name as a rule
        kb.add_rule_type(rule!("f", ["x"; instance!(sym!("Orange"))]));
        kb.add_rule(rule!("f", ["x"; instance!(sym!("Orange"))]));
        kb.add_rule(rule!("f", ["x"; instance!(sym!("Fruit"))]));

        let diagnostics = kb.validate_rules();
        assert_eq!(diagnostics.len(), 1);
        let diagnostic = diagnostics.into_iter().next().unwrap();
        let error = diagnostic.unwrap_error().unwrap_validation();
        assert!(matches!(error, InvalidRule { .. }));

        // Rule type does not apply if it doesn't have the same name as a rule
        kb.clear_rules();
        kb.add_rule_type(rule!("f", ["x"; instance!(sym!("Orange"))]));
        kb.add_rule(rule!("f", ["x"; instance!(sym!("Orange"))]));
        kb.add_rule(rule!("g", ["x"; instance!(sym!("Fruit"))]));
        assert!(kb.validate_rules().is_empty());

        // Rule type does apply if it has the same name as a rule even if different arity
        kb.clear_rules();
        kb.add_rule_type(rule!("f", ["x"; instance!(sym!("Orange")), value!(1)]));
        kb.add_rule(rule!("f", ["x"; instance!(sym!("Orange"))]));

        let diagnostic = kb.validate_rules().into_iter().next().unwrap();
        let error = diagnostic.unwrap_error().unwrap_validation();
        assert!(matches!(error, InvalidRule { .. }));

        // Multiple templates can exist for the same name but only one needs to match
        kb.clear_rules();
        kb.add_rule_type(rule!("f", ["x"; instance!(sym!("Orange"))]));
        kb.add_rule_type(rule!("f", ["x"; instance!(sym!("Orange")), value!(1)]));
        kb.add_rule_type(rule!("f", ["x"; instance!(sym!("Fruit"))]));
        kb.add_rule(rule!("f", ["x"; instance!(sym!("Fruit"))]));
        assert!(kb.validate_rules().is_empty());
    }

    #[test]
    fn test_rule_type_validation_errors_for_non_class_specializers() {
        let mut kb = KnowledgeBase::new();

        kb.register_constant(sym!("String1"), term!("not an external instance"))
            .unwrap();
        kb.register_constant(sym!("String2"), term!("also not an external instance"))
            .unwrap();
        kb.register_constant(
            sym!("ExternalInstanceWithoutMRO1"),
            term!(Value::ExternalInstance(ExternalInstance {
                instance_id: 1,
                constructor: None,
                repr: None,
                class_repr: None,
                class_id: None,
            })),
        )
        .unwrap();
        kb.register_constant(
            sym!("ExternalInstanceWithoutMRO2"),
            term!(Value::ExternalInstance(ExternalInstance {
                instance_id: 2,
                constructor: None,
                repr: None,
                class_repr: None,
                class_id: None,
            })),
        )
        .unwrap();
        kb.register_constant(
            sym!("Class1"),
            term!(Value::ExternalInstance(ExternalInstance {
                instance_id: 3,
                constructor: None,
                repr: None,
                class_repr: None,
                class_id: None,
            })),
        )
        .unwrap();
        kb.add_mro(sym!("Class1"), vec![3]).unwrap();
        kb.register_constant(
            sym!("Class2"),
            term!(Value::ExternalInstance(ExternalInstance {
                instance_id: 4,
                constructor: None,
                repr: None,
                class_repr: None,
                class_id: None,
            })),
        )
        .unwrap();
        kb.add_mro(sym!("Class2"), vec![4]).unwrap();

        // Same unregistered specializer.
        kb.add_rule_type(rule!("f", ["_"; instance!("Unregistered")]));
        kb.add_rule(rule!("f", ["_"; instance!("Unregistered")]));
        assert!(kb.validate_rules().is_empty());

        // Different unregistered specializers.
        kb.clear_rules();
        kb.add_rule_type(rule!("f", ["_"; instance!("Unregistered1")]));
        kb.add_rule(rule!("f", ["_"; instance!("Unregistered2")]));
        let diagnostics = kb.validate_rules();
        assert_eq!(diagnostics.len(), 1);
        let diagnostic = diagnostics.first().unwrap().to_string();
        assert_eq!(diagnostic, "Unregistered class: Unregistered1");

        // Same specializer registered as a non-instance constant.
        kb.clear_rules();
        kb.add_rule_type(rule!("f", ["_"; instance!("String1")]));
        kb.add_rule(rule!("f", ["_"; instance!("String1")]));
        assert!(kb.validate_rules().is_empty());

        // Different specializers registered as non-instance constants.
        kb.clear_rules();
        kb.add_rule_type(rule!("f", ["_"; instance!("String1")]));
        kb.add_rule(rule!("f", ["_"; instance!("String2")]));
        let diagnostics = kb.validate_rules();
        assert_eq!(diagnostics.len(), 1);
        let diagnostic = diagnostics.first().unwrap().to_string();
        let expected = "Rule type specializer String1 on parameter 1 should be a registered class, but instead it's registered as a constant with value: \"not an external instance\"";
        assert!(diagnostic.contains(expected), "{}", diagnostic);

        // Same specializer registered as an external instance without an MRO.
        kb.clear_rules();
        kb.add_rule_type(rule!("f", ["_"; instance!("ExternalInstanceWithoutMRO1")]));
        kb.add_rule(rule!("f", ["_"; instance!("ExternalInstanceWithoutMRO1")]));
        assert!(kb.validate_rules().is_empty());

        // Different specializers registered as external instances without MROs.
        kb.clear_rules();
        kb.add_rule_type(rule!("f", ["_"; instance!("ExternalInstanceWithoutMRO1")]));
        kb.add_rule(rule!("f", ["_"; instance!("ExternalInstanceWithoutMRO2")]));
        let diagnostics = kb.validate_rules();
        assert_eq!(diagnostics.len(), 1);
        let diagnostic = diagnostics.first().unwrap().to_string();
        let expected = "Rule specializer ExternalInstanceWithoutMRO2 on parameter 1 is not registered as a class.";
        assert!(diagnostic.contains(expected), "{}", diagnostic);

        // Same specializer registered as a class.
        kb.clear_rules();
        kb.add_rule_type(rule!("f", ["_"; instance!("Class1")]));
        kb.add_rule(rule!("f", ["_"; instance!("Class1")]));
        assert!(kb.validate_rules().is_empty());

        // Different specializers registered as classes.
        kb.clear_rules();
        kb.add_rule_type(rule!("f", ["_"; instance!("Class1")]));
        kb.add_rule(rule!("f", ["_"; instance!("Class2")]));
        let diagnostics = kb.validate_rules();
        assert_eq!(diagnostics.len(), 1);
        let diagnostic = diagnostics.first().unwrap().to_string();
        let expected =
            "Rule specializer Class2 on parameter 1 must match rule type specializer Class1";
        assert!(diagnostic.contains(expected), "{}", diagnostic);

        // Rule type specializer: unregistered
        // Rule specializer: non-instance constant
        kb.clear_rules();
        kb.add_rule_type(rule!("f", ["_"; instance!("Unregistered")]));
        kb.add_rule(rule!("f", ["_"; instance!("String1")]));
        let diagnostics = kb.validate_rules();
        assert_eq!(diagnostics.len(), 1);
        let diagnostic = diagnostics.first().unwrap().to_string();
        assert_eq!(diagnostic, "Unregistered class: Unregistered");

        // Rule type specializer: non-instance constant
        // Rule specializer: unregistered
        kb.clear_rules();
        kb.add_rule_type(rule!("f", ["_"; instance!("String1")]));
        kb.add_rule(rule!("f", ["_"; instance!("Unregistered")]));
        let diagnostics = kb.validate_rules();
        assert_eq!(diagnostics.len(), 1);
        let diagnostic = diagnostics.first().unwrap().to_string();
        let expected = "Rule type specializer String1 on parameter 1 should be a registered class, but instead it's registered as a constant with value: \"not an external instance\"";
        assert!(diagnostic.contains(expected), "{}", diagnostic);

        // Rule type specializer: external instance w/o MRO
        // Rule specializer: unregistered
        kb.clear_rules();
        kb.add_rule_type(rule!("f", ["_"; instance!("ExternalInstanceWithoutMRO1")]));
        kb.add_rule(rule!("f", ["_"; instance!("Unregistered")]));
        let diagnostics = kb.validate_rules();
        assert_eq!(diagnostics.len(), 1);
        let diagnostic = diagnostics.first().unwrap().to_string();
        let expected = "Rule specializer Unregistered on parameter 1 is not registered as a class.";
        assert!(diagnostic.contains(expected), "{}", diagnostic);

        // Rule type specializer: external instance w/o MRO
        // Rule specializer: class
        kb.clear_rules();
        kb.add_rule_type(rule!("f", ["_"; instance!("ExternalInstanceWithoutMRO1")]));
        kb.add_rule(rule!("f", ["_"; instance!("Class1")]));
        let diagnostics = kb.validate_rules();
        assert_eq!(diagnostics.len(), 1);
        let diagnostic = diagnostics.first().unwrap().to_string();
        let expected = "Rule specializer Class1 on parameter 1 must match rule type specializer ExternalInstanceWithoutMRO1";
        assert!(diagnostic.contains(expected), "{}", diagnostic);
    }
}
