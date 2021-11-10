use std::sync::{Arc, RwLock};

use super::data_filtering::{build_filter_plan, FilterPlan, PartialResults, Types};
use super::diagnostic::{set_context_for_diagnostics, Diagnostic};
use super::error::PolarResult;
use super::kb::*;
use super::messages::*;
use super::parser;
use super::query::Query;
use super::resource_block::resource_block_from_productions;
use super::rewrites::*;
use super::sources::*;
use super::terms::*;
use super::validations::{
    check_ambiguous_precedence, check_no_allow_rule, check_resource_blocks_missing_has_permission,
    check_singletons,
};
use super::vm::*;

pub struct Polar {
    pub kb: Arc<RwLock<KnowledgeBase>>,
    messages: MessageQueue,
    ignore_no_allow_warning: bool,
}

impl Default for Polar {
    fn default() -> Self {
        Self::new()
    }
}

const MULTIPLE_LOAD_ERROR_MSG: &str =
    "Cannot load additional Polar code -- all Polar code must be loaded at the same time.";

impl Polar {
    pub fn new() -> Self {
        // TODO(@gkaemmer): pulling this from an environment variable is a hack
        // and should not be used for similar cases. See set_ignore_no_allow_warning.
        // Ideally, we'd have a single "configuration" entrypoint for both the Polar
        // and Query types, so that we don't have to keep adding environment
        // variables for new configuration use-cases.
        let ignore_no_allow_warning = std::env::var("POLAR_IGNORE_NO_ALLOW_WARNING").is_ok();
        Self {
            kb: Arc::new(RwLock::new(KnowledgeBase::new())),
            messages: MessageQueue::new(),
            ignore_no_allow_warning,
        }
    }

    /// Load `sources` into the KB, returning compile-time diagnostics accumulated during the load.
    pub fn diagnostic_load(&self, sources: Vec<Source>) -> Vec<Diagnostic> {
        // we extract this into a separate function
        // so that any errors returned with `?` are captured
        fn load_source(
            source_id: u64,
            source: &Source,
            kb: &mut KnowledgeBase,
        ) -> PolarResult<Vec<Diagnostic>> {
            // TODO(gj): we still bomb out at the first ParseError.
            let mut lines = parser::parse_lines(source_id, &source.src)?;
            lines.reverse();
            let mut diagnostics = vec![];
            while let Some(line) = lines.pop() {
                match line {
                    parser::Line::Rule(rule) => {
                        diagnostics.append(&mut check_singletons(&rule, kb));
                        diagnostics.append(&mut check_ambiguous_precedence(&rule, kb));
                        let rule = rewrite_rule(rule, kb);
                        kb.add_rule(rule);
                    }
                    parser::Line::Query(term) => {
                        kb.inline_queries.push(term);
                    }
                    parser::Line::RuleType(rule_type) => {
                        // make sure rule_type doesn't have anything that needs to be rewritten in the head
                        let rule_type = rewrite_rule(rule_type, kb);
                        if !matches!(
                            rule_type.body.value(),
                            Value::Expression(
                                Operation {
                                    operator: Operator::And,
                                    args
                                }
                            ) if args.is_empty()
                        ) {
                            diagnostics.push(Diagnostic::Error(
                                error::ValidationError::InvalidRuleType {
                                    rule_type,
                                    msg: "Rule types cannot contain dot lookups.".to_owned(),
                                }
                                .into(),
                            ));
                        } else {
                            kb.add_rule_type(rule_type);
                        }
                    }
                    parser::Line::ResourceBlock {
                        keyword,
                        resource,
                        productions,
                    } => match resource_block_from_productions(keyword, resource, productions)
                        .map(|block| block.add_to_kb(kb))
                    {
                        Ok(errors) | Err(errors) => diagnostics
                            .append(&mut errors.into_iter().map(Diagnostic::Error).collect()),
                    },
                }
            }
            Ok(diagnostics)
        }

        let mut kb = self.kb.write().unwrap();
        let mut diagnostics = vec![];

        for source in &sources {
            let result = kb.add_source(source.clone());
            let result = result.and_then(|source_id| load_source(source_id, source, &mut kb));
            match result {
                Ok(mut ds) => diagnostics.append(&mut ds),
                Err(e) => diagnostics.push(Diagnostic::Error(e)),
            }
        }

        // Rewrite shorthand rules in resource blocks before validating rule types.
        diagnostics.append(
            &mut kb
                .rewrite_shorthand_rules()
                .into_iter()
                .map(Diagnostic::Error)
                .collect(),
        );

        // TODO(gj): need to bomb out before rule type validation in case additional rule types
        // were defined later on in the file that encountered the `ParseError`. Those additional
        // rule types might extend the valid shapes for a rule type defined in a different,
        // well-parsed file that also contains rules that don't conform to the shapes laid out in
        // the well-parsed file but *would have* conformed to the shapes laid out in the file that
        // failed to parse.
        if diagnostics.iter().any(Diagnostic::is_parse_error) {
            // NOTE(gj): need to set context _before_ clearing the KB so we still have source info.
            set_context_for_diagnostics(&kb, &mut diagnostics);
            kb.clear_rules();
            return diagnostics;
        }

        // Generate appropriate rule_type definitions using the types contained in policy resource
        // blocks.
        kb.create_resource_specific_rule_types();

        // check rules are valid against rule types
        diagnostics.append(&mut kb.validate_rules());

        // Perform validation checks against the whole policy
        if !self.ignore_no_allow_warning {
            if let Some(w) = check_no_allow_rule(&kb) {
                diagnostics.push(w)
            }
        }

        // Check for has_permission calls alongside resource block definitions
        if let Some(w) = check_resource_blocks_missing_has_permission(&kb) {
            diagnostics.push(w)
        };

        // NOTE(gj): need to set context _before_ clearing the KB so we still have source info.
        set_context_for_diagnostics(&kb, &mut diagnostics);

        // If we've encountered any errors, clear the KB.
        if diagnostics.iter().any(Diagnostic::is_error) {
            kb.clear_rules();
        }

        diagnostics
    }

    /// Load `Source`s into the KB.
    pub fn load(&self, sources: Vec<Source>) -> PolarResult<()> {
        if self.kb.read().unwrap().has_rules() {
            let msg = MULTIPLE_LOAD_ERROR_MSG.to_owned();
            return Err(error::RuntimeError::FileLoading { msg }.into());
        }

        let (mut errors, mut warnings) = (vec![], vec![]);
        for diagnostic in self.diagnostic_load(sources) {
            match diagnostic {
                Diagnostic::Error(e) => errors.push(e),
                Diagnostic::Warning(w) => warnings.push(w),
            }
        }

        self.messages
            .extend(warnings.into_iter().map(Message::warning));

        if let Some(e) = errors.into_iter().next() {
            return Err(e);
        }
        Ok(())
    }

    // Used in integration tests
    pub fn load_str(&self, src: &str) -> PolarResult<()> {
        self.load(vec![Source {
            src: src.to_owned(),
            filename: None,
        }])
    }

    /// Clear rules from the knowledge base
    pub fn clear_rules(&self) {
        let mut kb = self.kb.write().unwrap();
        kb.clear_rules();
    }

    pub fn next_inline_query(&self, trace: bool) -> Option<Query> {
        let term = { self.kb.write().unwrap().inline_queries.pop() };
        term.map(|t| self.new_query_from_term(t, trace))
    }

    pub fn new_query(&self, src: &str, trace: bool) -> PolarResult<Query> {
        let source = Source {
            filename: None,
            src: src.to_owned(),
        };
        let term = {
            let mut kb = self.kb.write().unwrap();
            let src_id = kb.new_id();
            let term =
                parser::parse_query(src_id, src).map_err(|e| e.set_context(Some(&source), None))?;
            kb.sources.add_source(source, src_id);
            term
        };
        Ok(self.new_query_from_term(term, trace))
    }

    pub fn new_query_from_term(&self, mut term: Term, trace: bool) -> Query {
        {
            let mut kb = self.kb.write().unwrap();
            term = rewrite_term(term, &mut kb);
        }
        let query = Goal::Query { term: term.clone() };
        let vm =
            PolarVirtualMachine::new(self.kb.clone(), trace, vec![query], self.messages.clone());
        Query::new(vm, term)
    }

    // @TODO: Direct load_rules endpoint.

    pub fn get_external_id(&self) -> u64 {
        self.kb.read().unwrap().new_id()
    }

    pub fn register_constant(&self, name: Symbol, value: Term) -> PolarResult<()> {
        self.kb.write().unwrap().register_constant(name, value)
    }

    /// Register MRO for `name` with `mro`.
    ///
    /// Params:
    ///
    /// - `mro`: Should go from `name`, `name`'s next superclass, `name's furthest away superclass.
    ///          `mro` is a list of class ids.
    pub fn register_mro(&self, name: Symbol, mro: Vec<u64>) -> PolarResult<()> {
        self.kb.write().unwrap().add_mro(name, mro)
    }

    pub fn next_message(&self) -> Option<Message> {
        self.messages.next()
    }

    pub fn build_filter_plan(
        &self,
        types: Types,
        partial_results: PartialResults,
        variable: &str,
        class_tag: &str,
    ) -> PolarResult<FilterPlan> {
        build_filter_plan(types, partial_results, variable, class_tag)
    }

    // TODO(@gkaemmer): this is a hack and should not be used for similar cases.
    // Ideally, we'd have a single "configuration" entrypoint for both the Polar
    // and Query types.
    pub fn set_ignore_no_allow_warning(&mut self, ignore: bool) {
        self.ignore_no_allow_warning = ignore;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_load_and_query() {
        let polar = Polar::new();
        let _query = polar.new_query("1 = 1", false);
        let _ = polar.load_str("f(_);");
    }

    #[test]
    fn loading_a_second_time_fails() {
        let polar = Polar::new();
        let src = "f();";
        let source = Source {
            src: src.to_owned(),
            filename: None,
        };

        // Loading once is fine.
        polar.load(vec![source.clone()]).unwrap();

        // Loading twice is not.
        let msg = match polar.load(vec![source]).unwrap_err() {
            error::PolarError {
                kind: error::ErrorKind::Runtime(error::RuntimeError::FileLoading { msg }),
                ..
            } => msg,
            e => panic!("{}", e),
        };
        assert_eq!(msg, MULTIPLE_LOAD_ERROR_MSG);

        // Even with load_str().
        let msg = match polar.load_str(src).unwrap_err() {
            error::PolarError {
                kind: error::ErrorKind::Runtime(error::RuntimeError::FileLoading { msg }),
                ..
            } => msg,
            e => panic!("{}", e),
        };
        assert_eq!(msg, MULTIPLE_LOAD_ERROR_MSG);
    }

    #[test]
    fn loading_duplicate_files_errors_and_leaves_the_kb_empty() {
        let polar = Polar::new();
        let source = Source {
            src: "f();".to_owned(),
            filename: Some("file".to_owned()),
        };

        let msg = match polar.load(vec![source.clone(), source]).unwrap_err() {
            error::PolarError {
                kind: error::ErrorKind::Runtime(error::RuntimeError::FileLoading { msg }),
                ..
            } => msg,
            e => panic!("{}", e),
        };
        assert_eq!(msg, "File file has already been loaded.");

        assert!(!polar.kb.read().unwrap().has_rules());
    }

    #[test]
    fn diagnostic_load_returns_multiple_diagnostics() {
        let polar = Polar::new();
        let source = Source {
            src: "f() if g();".to_owned(),
            filename: Some("file".to_owned()),
        };

        let diagnostics = polar.diagnostic_load(vec![source]);
        assert_eq!(diagnostics.len(), 2);
        let mut diagnostics = diagnostics.into_iter();
        let next = diagnostics.next().unwrap();
        assert!(matches!(next, Diagnostic::Error(_)));
        assert!(
            next.to_string().starts_with("Call to undefined rule: g()"),
            "{}",
            next
        );
        let next = diagnostics.next().unwrap();
        assert!(matches!(next, Diagnostic::Warning(_)));
        assert!(
            next.to_string()
                .starts_with("Your policy does not contain an allow rule"),
            "{}",
            next
        );
        assert!(!polar.kb.read().unwrap().has_rules());
    }
}
