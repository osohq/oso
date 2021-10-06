use std::collections::{HashMap, HashSet};

use crate::error::ParameterError;
use crate::error::{PolarError, PolarResult};

pub use super::bindings::Bindings;
use super::counter::Counter;
use super::resource_block::ResourceBlocks;
use super::resource_block::{ACTOR_UNION_NAME, RESOURCE_UNION_NAME};
use super::rules::*;
use super::sources::*;
use super::terms::*;
use std::sync::Arc;

enum RuleParamMatch {
    True,
    False(String),
}

impl RuleParamMatch {
    #[cfg(test)]
    fn is_true(&self) -> bool {
        matches!(self, RuleParamMatch::True)
    }
}

#[derive(Default)]
pub struct KnowledgeBase {
    /// A map of bindings: variable name → value. The VM uses a stack internally,
    /// but can translate to and from this type.
    pub constants: Bindings,
    /// Map of class name -> MRO list where the MRO list is a list of class instance IDs
    mro: HashMap<Symbol, Vec<u64>>,

    /// Map from loaded files to the source ID
    pub loaded_files: HashMap<String, u64>,
    /// Map from source code loaded to the filename it was loaded as
    pub loaded_content: HashMap<String, String>,

    rules: HashMap<Symbol, GenericRule>,
    rule_types: RuleTypes,
    pub sources: Sources,
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
        Self {
            constants: HashMap::new(),
            mro: HashMap::new(),
            loaded_files: Default::default(),
            loaded_content: Default::default(),
            rules: HashMap::new(),
            rule_types: RuleTypes::default(),
            sources: Sources::default(),
            id_counter: Counter::default(),
            gensym_counter: Counter::default(),
            inline_queries: vec![],
            resource_blocks: ResourceBlocks::new(),
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

    pub fn add_rule(&mut self, rule: Rule) {
        let generic_rule = self
            .rules
            .entry(rule.name.clone())
            .or_insert_with(|| GenericRule::new(rule.name.clone(), vec![]));
        generic_rule.add_rule(Arc::new(rule));
    }

    /// Validate that all rules loaded into the knowledge base are valid based on rule types.
    pub fn validate_rules(&self) -> PolarResult<()> {
        for (rule_name, generic_rule) in &self.rules {
            if let Some(types) = self.rule_types.get(rule_name) {
                // If a type with the same name exists, then the parameters must match for each rule
                for rule in generic_rule.rules.values() {
                    let mut msg = "Must match one of the following rule types:\n".to_owned();

                    let found_match = types
                        .iter()
                        .map(|rule_type| {
                            self.rule_params_match(rule.as_ref(), rule_type)
                                .map(|result| (result, rule_type))
                        })
                        .collect::<PolarResult<Vec<(RuleParamMatch, &Rule)>>>()
                        .map(|results| {
                            results.iter().any(|(result, rule_type)| match result {
                                RuleParamMatch::True => true,
                                RuleParamMatch::False(message) => {
                                    msg.push_str(&format!(
                                        "\n{}\n\tFailed to match because: {}\n",
                                        rule_type.to_polar(),
                                        message
                                    ));
                                    false
                                }
                            })
                        })?;
                    if !found_match {
                        return Err(self.set_error_context(
                            &rule.body,
                            error::ValidationError::InvalidRule {
                                rule: rule.to_polar(),
                                msg,
                            },
                        ));
                    }
                }
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
        if let Some(Value::ExternalInstance(ExternalInstance { instance_id, .. })) = self
            .constants
            .get(&rule_type_instance.tag)
            .map(|t| t.value())
        {
            if let Some(rule_mro) = self.mro.get(&rule_instance.tag) {
                if !rule_mro.contains(instance_id) {
                    Ok(RuleParamMatch::False(format!(
                        "Rule specializer {} on parameter {} must match rule type specializer {}",
                        rule_instance.tag, index, rule_type_instance.tag
                    )))
                } else if !self
                    .param_fields_match(&rule_type_instance.fields, &rule_instance.fields)
                {
                    Ok(RuleParamMatch::False(format!("Rule specializer {} on parameter {} did not match rule type specializer {} because the specializer fields did not match.", rule_instance.to_polar(), index, rule_type_instance.to_polar())))
                } else {
                    Ok(RuleParamMatch::True)
                }
            } else {
                Err(error::OperationalError::InvalidState{msg: format!(
                    "All registered classes must have a registered MRO. Class {} does not have a registered MRO.",
                    &rule_instance.tag
                )}.into())
            }
        } else {
            unreachable!("Unregistered specializer classes should be caught before this point.");
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
                        RuleParamMatch::False(format!("Rule specializer {} on parameter {} did not match rule type specializer {} because the specializer fields did not match.", rule_instance.to_polar(), index, rule_type_instance.to_polar()))
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
                                return Ok(RuleParamMatch::False(format!("Rule specializer {} on parameter {} did not match rule type specializer {} because the specializer fields did not match.", rule_instance.to_polar(), index, rule_type_instance.to_polar())));
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
                                tag: member.value().as_symbol()?.clone(),
                                fields: rule_type_instance.fields.clone()
                            };
                            match self.check_rule_instance_is_subclass_of_rule_type_instance(rule_instance, &rule_type_instance, index) {
                                Ok(RuleParamMatch::True) if !success => success = true,
                                Err(e) => return Err(e),
                                _ => (),
                            }
                        }
                        if !success {
                            if rule_type_instance.tag == sym!("Actor") {
                                return Ok(RuleParamMatch::False(format!("Rule specializer {} on parameter {} must be a member of rule type specializer {}

    Perhaps you meant to add an actor block to the top of your policy, like this:

    actor {} {{}}",
                                rule_instance.tag, index, rule_type_instance.tag, rule_instance.tag)));

                            } else {
                                return Ok(RuleParamMatch::False(format!("Rule specializer {} on parameter {} must be a member of rule type specializer {}", rule_instance.tag,index, rule_type_instance.tag)));
                            }

                        }
                    }
                    if !self.param_fields_match(&rule_type_instance.fields, &rule_instance.fields) {
                        RuleParamMatch::False(format!("Rule specializer {} on parameter {} did not match rule type specializer {} because the specializer fields did not match.", rule_instance.to_polar(), index, rule_type_instance.to_polar()))
                    } else {
                        RuleParamMatch::True
                    }
                // If tags don't match, then rule specializer must be a subclass of rule type specializer
                } else {
                    self.check_rule_instance_is_subclass_of_rule_type_instance(rule_instance, rule_type_instance, index)?
                }
            }
            (Pattern::Dictionary(rule_type_fields), Pattern::Dictionary(rule_fields))
            | (
                Pattern::Dictionary(rule_type_fields),
                Pattern::Instance(InstanceLiteral {
                    tag: _,
                    fields: rule_fields,
                }),
            ) => {
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
    ) -> PolarResult<RuleParamMatch> {
        Ok(match (rule_type_value, rule_value) {
            // List in rule head must be equal to or more specific than the list in the rule type head in order to match
            (Value::List(rule_type_list), Value::List(rule_list)) => {
                if has_rest_var(rule_type_list) {
                    return Err(error::RuntimeError::TypeError {
                        msg: "Rule types cannot contain *rest variables.".to_string(),
                        stack_trace: None,
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
                // RuleType has specializer but rule doesn't
                (Value::Variable(_), Some(rule_type_spec), Value::Variable(_), None) => {
                    RuleParamMatch::False(format!(
                        "Invalid rule parameter {}. Rule type expected {}",
                        index,
                        rule_type_spec.to_polar()
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
                                    unreachable!(
                                        "Value variant {} cannot be a specializer",
                                        rule_value
                                    )
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
                                    RuleParamMatch::False(format!("Invalid parameter {}. Rule type expected Dictionary with fields {}, got dictionary with fields {}.", index, rule_type_fields.to_polar(), rule_fields.to_polar()))
                                }
                            } else {
                                RuleParamMatch::False(format!(
                                    "Invalid parameter {}. Rule type expected Dictionary, got {}.",
                                    index,
                                    rule_value.to_polar()
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
                    self.check_value_param(index, rule_value, rule_type_value)?
                }
                _ => RuleParamMatch::False(format!(
                    "Invalid parameter {}. Rule parameter {} does not match rule type parameter {}",
                    index,
                    rule_param.to_polar(),
                    rule_type_param.to_polar()
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
                self.check_param(i + 1, rule_param, rule_type_param)
            })
            .collect::<PolarResult<Vec<RuleParamMatch>>>()
            .map(|results| {
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

    pub fn get_generic_rule(&self, name: &Symbol) -> Option<&GenericRule> {
        self.rules.get(name)
    }

    pub fn add_rule_type(&mut self, rule_type: Rule) {
        self.rule_types.add(rule_type);
    }

    /// Define a constant variable.
    pub fn constant(&mut self, name: Symbol, value: Term) -> PolarResult<()> {
        if name.0 == ACTOR_UNION_NAME || name.0 == RESOURCE_UNION_NAME {
            return Err(error::RuntimeError::TypeError {
                msg: format!(
                    "Invalid attempt to register '{}'. '{}' is a built-in specializer.",
                    name.0, name.0
                ),
                stack_trace: None,
            }
            .into());
        }
        self.constants.insert(name, value);
        Ok(())
    }

    /// Add the Method Resolution Order (MRO) list for a registered class.
    /// The `mro` argument is a list of the `instance_id` associated with a registered class.
    pub fn add_mro(&mut self, name: Symbol, mro: Vec<u64>) -> PolarResult<()> {
        // Confirm name is a registered class
        self.constants.get(&name).ok_or_else(|| {
            ParameterError(format!("Cannot add MRO for unregistered class {}", name))
        })?;
        self.mro.insert(name, mro);
        Ok(())
    }

    /// Return true if a constant with the given name has been defined.
    pub fn is_constant(&self, name: &Symbol) -> bool {
        self.constants.contains_key(name)
    }

    pub fn add_source(&mut self, source: Source) -> PolarResult<u64> {
        let src_id = self.new_id();
        if let Some(ref filename) = source.filename {
            self.check_file(&source.src, filename)?;
            self.loaded_content
                .insert(source.src.clone(), filename.to_string());
            self.loaded_files.insert(filename.to_string(), src_id);
        }
        self.sources.add_source(source, src_id);
        Ok(src_id)
    }

    pub fn clear_rules(&mut self) {
        self.rules.clear();
        self.rule_types.reset();
        self.sources = Sources::default();
        self.inline_queries.clear();
        self.loaded_content.clear();
        self.loaded_files.clear();
        self.resource_blocks.clear();
    }

    /// Removes a file from the knowledge base by finding the associated
    /// `Source` and removing all rules for that source, and
    /// removes the file from loaded files.
    ///
    /// Optionally return the source for the file, returning `None`
    /// if the file was not in the loaded files.
    pub fn remove_file(&mut self, filename: &str) -> Option<String> {
        self.loaded_files
            .get(filename)
            .cloned()
            .map(|src_id| self.remove_source(src_id))
    }

    /// Removes a source from the knowledge base by finding the associated
    /// `Source` and removing all rules for that source. Will
    /// also remove the loaded files if the source was loaded from a file.
    pub fn remove_source(&mut self, source_id: u64) -> String {
        // remove from rules
        self.rules.retain(|_, gr| {
            let to_remove: Vec<u64> = gr.rules.iter().filter_map(|(idx, rule)| {
                matches!(rule.source_info, SourceInfo::Parser { src_id, ..} if src_id == source_id)
                    .then(||*idx)
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
        let filename = source.filename;

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

    pub fn set_error_context(&self, term: &Term, error: impl Into<PolarError>) -> PolarError {
        let source = term
            .get_source_id()
            .and_then(|id| self.sources.get_source(id));
        let error: PolarError = error.into();
        error.set_context(source.as_ref(), Some(term))
    }

    pub fn rewrite_shorthand_rules(&mut self) -> PolarResult<()> {
        let mut errors = vec![];

        errors.append(
            &mut super::resource_block::check_all_relation_types_have_been_registered(self),
        );

        // TODO(gj): Emit all errors instead of just the first.
        if !errors.is_empty() {
            return Err(errors[0].clone());
        }

        let mut rules = vec![];
        for (resource_block, shorthand_rules) in &self.resource_blocks.shorthand_rules {
            for shorthand_rule in shorthand_rules {
                match shorthand_rule.as_rule(resource_block, &self.resource_blocks) {
                    Ok(rule) => rules.push(rule),
                    Err(error) => errors.push(error),
                }
            }
        }

        // TODO(gj): Emit all errors instead of just the first.
        if !errors.is_empty() {
            return Err(errors[0].clone());
        }

        // Add the rewritten rules to the KB.
        for rule in rules {
            self.add_rule(rule);
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
    use crate::error::*;

    #[test]
    /// Test validation implemented in `check_file()`.
    fn test_add_source_file_validation() {
        let mut kb = KnowledgeBase::new();
        let src = "f();";
        let filename1 = "f";
        let source1 = Source {
            src: src.to_owned(),
            filename: Some(filename1.to_owned()),
        };

        // Load source1.
        kb.add_source(source1.clone()).unwrap();

        // Cannot load source1 a second time.
        let msg = match kb.add_source(source1).unwrap_err() {
            error::PolarError {
                kind: error::ErrorKind::Runtime(error::RuntimeError::FileLoading { msg }),
                ..
            } => msg,
            e => panic!("{}", e),
        };
        assert_eq!(msg, format!("File {} has already been loaded.", filename1));

        // Cannot load source2 with the same name as source1 but different contents.
        let source2 = Source {
            src: "g();".to_owned(),
            filename: Some(filename1.to_owned()),
        };
        let msg = match kb.add_source(source2).unwrap_err() {
            error::PolarError {
                kind: error::ErrorKind::Runtime(error::RuntimeError::FileLoading { msg }),
                ..
            } => msg,
            e => panic!("{}", e),
        };
        assert_eq!(
            msg,
            format!(
                "A file with the name {}, but different contents has already been loaded.",
                filename1
            ),
        );

        // Cannot load source3 with the same contents as source1 but a different name.
        let filename2 = "g";
        let source3 = Source {
            src: src.to_owned(),
            filename: Some(filename2.to_owned()),
        };
        let msg = match kb.add_source(source3).unwrap_err() {
            error::PolarError {
                kind: error::ErrorKind::Runtime(error::RuntimeError::FileLoading { msg }),
                ..
            } => msg,
            e => panic!("{}", e),
        };
        assert_eq!(
            msg,
            format!(
                "A file with the same contents as {} named {} has already been loaded.",
                filename2, filename1
            ),
        );
    }

    #[test]
    fn test_rule_params_match() {
        let mut kb = KnowledgeBase::new();

        let mut constant = |name: &str, instance_id: u64| {
            kb.constant(
                sym!(name),
                term!(Value::ExternalInstance(ExternalInstance {
                    instance_id,
                    constructor: None,
                    repr: None
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
        kb.constant(
            sym!("Fruit"),
            term!(Value::ExternalInstance(ExternalInstance {
                instance_id: 1,
                constructor: None,
                repr: None
            })),
        )
        .unwrap();
        kb.constant(
            sym!("Citrus"),
            term!(Value::ExternalInstance(ExternalInstance {
                instance_id: 2,
                constructor: None,
                repr: None
            })),
        )
        .unwrap();
        kb.constant(
            sym!("Orange"),
            term!(Value::ExternalInstance(ExternalInstance {
                instance_id: 3,
                constructor: None,
                repr: None
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

        assert!(matches!(
            kb.validate_rules().err().unwrap(),
            PolarError {
                kind: ErrorKind::Validation(ValidationError::InvalidRule { .. }),
                ..
            }
        ));

        // Rule type does not apply if it doesn't have the same name as a rule
        kb.clear_rules();
        kb.add_rule_type(rule!("f", ["x"; instance!(sym!("Orange"))]));
        kb.add_rule(rule!("f", ["x"; instance!(sym!("Orange"))]));
        kb.add_rule(rule!("g", ["x"; instance!(sym!("Fruit"))]));

        kb.validate_rules().unwrap();

        // Rule type does apply if it has the same name as a rule even if different arity
        kb.clear_rules();
        kb.add_rule_type(rule!("f", ["x"; instance!(sym!("Orange")), value!(1)]));
        kb.add_rule(rule!("f", ["x"; instance!(sym!("Orange"))]));

        assert!(matches!(
            kb.validate_rules().err().unwrap(),
            PolarError {
                kind: ErrorKind::Validation(ValidationError::InvalidRule { .. }),
                ..
            }
        ));
        // Multiple templates can exist for the same name but only one needs to match
        kb.clear_rules();
        kb.add_rule_type(rule!("f", ["x"; instance!(sym!("Orange"))]));
        kb.add_rule_type(rule!("f", ["x"; instance!(sym!("Orange")), value!(1)]));
        kb.add_rule_type(rule!("f", ["x"; instance!(sym!("Fruit"))]));
        kb.add_rule(rule!("f", ["x"; instance!(sym!("Fruit"))]));
    }
}
