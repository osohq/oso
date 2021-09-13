use super::data_filtering::{build_filter_plan, FilterPlan, PartialResults, Types};
use super::error::PolarResult;
use super::events::*;
use super::kb::*;
use super::messages::*;
use super::parser;
use super::rewrites::*;
use super::runnable::Runnable;
use super::sources::*;
use super::terms::*;
use super::vm::*;
use super::warnings::{check_ambiguous_precedence, check_singletons};

use std::sync::{Arc, RwLock};

pub struct Query {
    runnable_stack: Vec<(Box<dyn Runnable>, u64)>, // Tuple of Runnable + call_id.
    vm: PolarVirtualMachine,
    term: Term,
    done: bool,
}

impl Query {
    pub fn new(vm: PolarVirtualMachine, term: Term) -> Self {
        Self {
            runnable_stack: vec![],
            vm,
            term,
            done: false,
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn set_logging_options(&mut self, rust_log: Option<String>, polar_log: Option<String>) {
        self.vm.set_logging_options(rust_log, polar_log);
    }

    /// Runnable lifecycle
    ///
    /// 1. Get Runnable A from the top of the Runnable stack, defaulting to the VM.
    /// 2. If Runnable A emits a Run event containing Runnable B, push Runnable B onto the stack.
    /// 3. Immediately request the next event, which will execute Runnable B.
    /// 4. When Runnable B emits a Done event, pop Runnable B off the stack and return its result as
    ///    an answer to Runnable A.
    pub fn next_event(&mut self) -> PolarResult<QueryEvent> {
        let mut counter = self.vm.id_counter();
        let qe = match self.top_runnable().run(Some(&mut counter)) {
            Ok(e) => e,
            Err(e) => self.top_runnable().handle_error(e)?,
        };
        self.recv_event(qe)
    }

    fn recv_event(&mut self, qe: QueryEvent) -> PolarResult<QueryEvent> {
        match qe {
            QueryEvent::None => self.next_event(),
            QueryEvent::Run { runnable, call_id } => {
                self.push_runnable(runnable, call_id);
                self.next_event()
            }
            QueryEvent::Done { result } => {
                if let Some((_, result_call_id)) = self.pop_runnable() {
                    self.top_runnable()
                        .external_question_result(result_call_id, result)?;
                    self.next_event()
                } else {
                    // VM is done.
                    assert!(self.runnable_stack.is_empty());
                    Ok(QueryEvent::Done { result })
                }
            }
            ev => Ok(ev),
        }
    }

    fn top_runnable(&mut self) -> &mut (dyn Runnable) {
        self.runnable_stack
            .last_mut()
            .map(|b| b.0.as_mut())
            .unwrap_or(&mut self.vm)
    }

    fn push_runnable(&mut self, runnable: Box<dyn Runnable>, call_id: u64) {
        self.runnable_stack.push((runnable, call_id));
    }

    fn pop_runnable(&mut self) -> Option<(Box<dyn Runnable>, u64)> {
        self.runnable_stack.pop()
    }

    pub fn call_result(&mut self, call_id: u64, value: Option<Term>) -> PolarResult<()> {
        self.top_runnable().external_call_result(call_id, value)
    }

    pub fn question_result(&mut self, call_id: u64, result: bool) -> PolarResult<()> {
        self.top_runnable()
            .external_question_result(call_id, result)
    }

    pub fn application_error(&mut self, message: String) -> PolarResult<()> {
        self.vm.external_error(message)
    }

    pub fn debug_command(&mut self, command: &str) -> PolarResult<()> {
        self.top_runnable().debug_command(command)
    }

    pub fn next_message(&self) -> Option<Message> {
        self.vm.messages.next()
    }

    pub fn source_info(&self) -> String {
        self.vm.term_source(&self.term, true)
    }

    pub fn bind(&mut self, name: Symbol, value: Term) -> PolarResult<()> {
        self.vm.bind(&name, value)
    }
}

// Query as an iterator returns `None` after the first time `Done` is seen
impl Iterator for Query {
    type Item = PolarResult<QueryEvent>;

    fn next(&mut self) -> Option<PolarResult<QueryEvent>> {
        if self.done {
            return None;
        }
        let event = self.next_event();
        if let Ok(QueryEvent::Done { .. }) = event {
            self.done = true;
        }
        Some(event)
    }
}

pub struct Polar {
    pub kb: Arc<RwLock<KnowledgeBase>>,
    messages: MessageQueue,
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
        Self {
            kb: Arc::new(RwLock::new(KnowledgeBase::new())),
            messages: MessageQueue::new(),
        }
    }

    /// Load `Source`s into the KB.
    pub fn load(&self, sources: Vec<Source>) -> PolarResult<()> {
        if self.kb.read().unwrap().has_rules() {
            let msg = MULTIPLE_LOAD_ERROR_MSG.to_owned();
            return Err(error::RuntimeError::FileLoading { msg }.into());
        }

        // we extract this into a separate function
        // so that any errors returned with `?` are captured
        fn load_source(
            source_id: u64,
            source: &Source,
            kb: &mut KnowledgeBase,
        ) -> PolarResult<Vec<String>> {
            let mut lines = parser::parse_lines(source_id, &source.src)
                .map_err(|e| e.set_context(Some(source), None))?;
            lines.reverse();
            let mut warnings = vec![];
            while let Some(line) = lines.pop() {
                match line {
                    parser::Line::Rule(rule) => {
                        let mut rule_warnings = check_singletons(&rule, &*kb)?;
                        warnings.append(&mut rule_warnings);
                        warnings.append(&mut check_ambiguous_precedence(&rule, &*kb)?);
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
                            return Err(kb.set_error_context(
                                &rule_type.body,
                                error::ValidationError::InvalidRuleType {
                                    rule_type: rule_type.to_polar(),
                                    msg: "\nRule types cannot contain dot lookups.".to_owned(),
                                },
                            ));
                        }
                        kb.add_rule_type(rule_type);
                    }
                    parser::Line::ResourceBlock(block) => {
                        block.add_to_kb(kb)?;
                    }
                }
            }
            Ok(warnings)
        }

        let mut kb = self.kb.write().unwrap();
        for source in &sources {
            let result = kb.add_source(source.clone());
            let result = result.and_then(|source_id| load_source(source_id, source, &mut kb));
            match result {
                Ok(warnings) => {
                    let warnings = warnings.into_iter().map(|msg| Message {
                        kind: MessageKind::Warning,
                        msg,
                    });
                    self.messages.extend(warnings);
                }
                Err(e) => {
                    // If any source fails to load, clear the KB.
                    kb.clear_rules();
                    return Err(e);
                }
            }
        }

        // Rewrite shorthand rules in resource blocks before validating rule types.
        if let Err(e) = kb.rewrite_shorthand_rules() {
            // If rewriting shorthand rules fails, clear the KB.
            kb.clear_rules();
            return Err(e);
        }

        // check rules are valid against rule types
        if let Err(e) = kb.validate_rules() {
            // If rule type validation fails, clear the KB.
            kb.clear_rules();
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

    // TODO(gj): ask Sam if we still need this.
    pub fn remove_file(&self, filename: &str) -> Option<String> {
        let mut kb = self.kb.write().unwrap();
        kb.remove_file(filename)
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
        self.kb.write().unwrap().constant(name, value)
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
    fn load_remove_files() {
        let polar = Polar::new();
        let valid = Source {
            src: "f(x) if x = 1;".to_owned(),
            filename: Some("test.polar".to_string()),
        };
        polar.load(vec![valid.clone()]).unwrap();
        polar.remove_file("test.polar");

        // loading works after removing
        polar.load(vec![valid.clone()]).unwrap();
        polar.remove_file("test.polar");

        // load a broken file
        let invalid = Source {
            src: "f(x) if x".to_owned(),
            filename: Some("test.polar".to_string()),
        };
        polar.load(vec![invalid]).unwrap_err();

        // can still load files again
        polar.load(vec![valid]).unwrap();
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
    fn loading_duplicate_files_clears_the_kb() {
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
}
