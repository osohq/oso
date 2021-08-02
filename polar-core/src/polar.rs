use super::error::PolarResult;
use super::events::*;
use super::kb::*;
use super::messages::*;
use super::parser;
use super::rewrites::*;
use super::roles_validation::{
    validate_roles_config, ResultEvent, VALIDATE_ROLES_CONFIG_RESOURCES,
};
use super::rules::*;
use super::runnable::Runnable;
use super::sources::*;
use super::terms::*;
use super::vm::*;
use super::warnings::check_singletons;

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

const ROLES_POLICY: &str = include_str!("roles.polar");

pub struct Polar {
    pub kb: Arc<RwLock<KnowledgeBase>>,
    messages: MessageQueue,
}

impl Default for Polar {
    fn default() -> Self {
        Self::new()
    }
}

impl Polar {
    pub fn new() -> Self {
        Self {
            kb: Arc::new(RwLock::new(KnowledgeBase::new())),
            messages: MessageQueue::new(),
        }
    }

    pub fn load(&self, src: &str, filename: Option<String>) -> PolarResult<()> {
        let source = Source {
            filename,
            src: src.to_owned(),
        };
        let mut kb = self.kb.write().unwrap();
        let source_id = kb.add_source(source.clone())?;

        // extract this into a seperate function to we can catch all errors
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
                        let rule = rewrite_rule(rule, kb);

                        let name = rule.name.clone();
                        let generic_rule = kb
                            .rules
                            .entry(name.clone())
                            .or_insert_with(|| GenericRule::new(name, vec![]));
                        generic_rule.add_rule(Arc::new(rule));
                    }
                    parser::Line::Query(term) => {
                        kb.inline_queries.push(term);
                    }
                }
            }
            Ok(warnings)
        }

        // if any of the lines fail to load, we need to remove the source from
        // the knowledge base
        match load_source(source_id, &source, &mut kb) {
            Ok(warnings) => {
                self.messages.extend(warnings.iter().map(|m| Message {
                    kind: MessageKind::Warning,
                    msg: m.to_owned(),
                }));
                Ok(())
            }
            Err(e) => {
                kb.remove_source(source.filename, source_id);
                Err(e)
            }
        }
    }

    // Used in integration tests
    pub fn load_str(&self, src: &str) -> PolarResult<()> {
        self.load(src, None)
    }

    pub fn remove_file(&self, filename: &str) -> Option<String> {
        let mut kb = self.kb.write().unwrap();
        kb.remove_file(filename)
    }

    /// Clear rules from the knowledge base
    pub fn clear_rules(&self) {
        let mut kb = self.kb.write().unwrap();
        kb.clear_sources();
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

    pub fn register_constant(&self, name: Symbol, value: Term) {
        self.kb.write().unwrap().constant(name, value)
    }

    pub fn next_message(&self) -> Option<Message> {
        self.messages.next()
    }

    /// Load the Polar roles policy idempotently.
    pub fn enable_roles(&self) -> PolarResult<()> {
        let result = match self.load(ROLES_POLICY, Some("Built-in Polar Roles Policy".to_owned())) {
            Err(error::PolarError {
                kind: error::ErrorKind::Runtime(error::RuntimeError::FileLoading { .. }),
                ..
            }) => Ok(()),
            result => result,
        };

        // Push inline queries to validate config.
        let src_id = self.kb.read().unwrap().new_id();
        let term = parser::parse_query(src_id, VALIDATE_ROLES_CONFIG_RESOURCES)?;
        self.kb.write().unwrap().inline_queries.push(term);

        result
    }

    pub fn validate_roles_config(&self, results: Vec<Vec<ResultEvent>>) -> PolarResult<()> {
        validate_roles_config(&self.kb.read().unwrap().rules, results)
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
    fn roles_policy_loads_idempotently() {
        let polar = Polar::new();
        assert!(polar.enable_roles().is_ok());
        {
            let kb = polar.kb.read().unwrap();
            assert_eq!(kb.loaded_files.len(), 1);
            assert_eq!(kb.loaded_content.len(), 1);
        }
        assert!(polar.enable_roles().is_ok());
        {
            let kb = polar.kb.read().unwrap();
            assert_eq!(kb.loaded_files.len(), 1);
            assert_eq!(kb.loaded_content.len(), 1);
        }
    }

    #[test]
    fn load_remove_files() {
        let polar = Polar::new();
        polar
            .load("f(x) if x = 1;", Some("test.polar".to_string()))
            .unwrap();
        polar.remove_file("test.polar");
        // loading works after removing
        polar
            .load("f(x) if x = 1;", Some("test.polar".to_string()))
            .unwrap();
        polar.remove_file("test.polar");

        // load a broken file
        polar
            .load("f(x) if x", Some("test.polar".to_string()))
            .unwrap_err();

        // can still load files again
        polar
            .load("f(x) if x = 1;", Some("test.polar".to_string()))
            .unwrap();
    }
}
