use super::error::PolarResult;
use super::events::*;
use super::kb::*;
use super::messages::*;
use super::parser;
use super::rewrites::*;
use super::rules::*;
use super::runnable::Runnable;
use super::sources::*;
use super::terms::*;
use super::vm::*;
use super::warnings::check_singletons;

use std::collections::{HashMap, HashSet};
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

    /// Runnable lifecycle
    ///
    /// 1. Get Runnable A from the top of the Runnable stack, defaulting to the VM.
    /// 2. If Runnable A emits a Run event containing Runnable B, push Runnable B onto the stack.
    /// 3. Immediately request the next event, which will execute Runnable B.
    /// 4. When Runnable B emits a Done event, pop Runnable B off the stack and return its result as
    ///    an answer to Runnable A.
    pub fn next_event(&mut self) -> PolarResult<QueryEvent> {
        let mut counter = self.vm.id_counter();
        let runnable = self.runnable_stack.last_mut();
        let bindings = &mut self.vm.bindings;
        let result = if let Some(runnable) = runnable {
            let i = self.vm.choices.len() - 2; // Noop for Not
            let bsp = self.vm.choices.get_mut(i).map(|c| &mut c.bsp);
            runnable
                .0
                .as_mut()
                .run(Some(bindings), bsp, Some(&mut counter))
        } else {
            self.vm.run(None, None, None)
        };

        match result? {
            QueryEvent::Run { runnable, call_id } => {
                self.push_runnable(runnable, call_id)?;
                self.next_event()
            }
            QueryEvent::Done { result } => {
                if let Some((_, result_call_id)) = self.pop_runnable() {
                    let runnable = if let Some(runnable) = self.top_runnable() {
                        runnable
                    } else {
                        &mut self.vm
                    };
                    runnable.external_question_result(result_call_id, result)?;
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

    fn top_runnable(&mut self) -> Option<&mut (dyn Runnable + 'static)> {
        self.runnable_stack.last_mut().map(|b| b.0.as_mut())
    }

    fn push_runnable(&mut self, runnable: Box<dyn Runnable>, call_id: u64) -> PolarResult<()> {
        self.runnable_stack.push((runnable, call_id));
        Ok(())
    }

    fn pop_runnable(&mut self) -> Option<(Box<dyn Runnable>, u64)> {
        self.runnable_stack.pop()
    }

    pub fn call_result(&mut self, call_id: u64, value: Option<Term>) -> PolarResult<()> {
        if let Some(runnable) = self.top_runnable() {
            return runnable.external_call_result(call_id, value);
        }
        self.vm.external_call_result(call_id, value)
    }

    pub fn question_result(&mut self, call_id: u64, result: bool) -> PolarResult<()> {
        if let Some(runnable) = self.top_runnable() {
            return runnable.external_question_result(call_id, result);
        }
        self.vm.external_question_result(call_id, result)
    }

    pub fn application_error(&mut self, message: String) -> PolarResult<()> {
        if let Some(runnable) = self.top_runnable() {
            return runnable.external_error(message);
        }
        self.vm.external_error(message)
    }

    pub fn debug_command(&mut self, command: &str) -> PolarResult<()> {
        self.vm.debug_command(command)
    }

    pub fn next_message(&self) -> Option<Message> {
        self.vm.messages.next()
    }

    pub fn source_info(&self) -> String {
        self.vm.term_source(&self.term, true)
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
    /// Set of filenames already loaded
    loaded_files: Arc<RwLock<HashSet<String>>>,
    /// Map from source code loaded to the filename it was loaded as
    loaded_content: Arc<RwLock<HashMap<String, String>>>,
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
            loaded_content: Arc::new(RwLock::new(HashMap::new())), // file content -> file name
            loaded_files: Arc::new(RwLock::new(HashSet::new())),   // set of file names
        }
    }

    fn check_file(&self, src: &str, filename: &str) -> PolarResult<()> {
        match (
            self.loaded_content.read().unwrap().get(src),
            self.loaded_files.read().unwrap().contains(filename),
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
        self.loaded_content
            .write()
            .unwrap()
            .insert(src.to_string(), filename.to_string());
        self.loaded_files
            .write()
            .unwrap()
            .insert(filename.to_string());

        Ok(())
    }

    pub fn load(&self, src: &str, filename: Option<String>) -> PolarResult<()> {
        if let Some(ref filename) = filename {
            self.check_file(src, filename)?;
        }
        let source = Source {
            filename,
            src: src.to_owned(),
        };
        let mut kb = self.kb.write().unwrap();
        let src_id = kb.new_id();
        let mut lines =
            parser::parse_lines(src_id, src).map_err(|e| e.set_context(Some(&source), None))?;
        lines.reverse();
        kb.sources.add_source(source, src_id);
        let mut warnings = vec![];
        while let Some(line) = lines.pop() {
            match line {
                parser::Line::Rule(rule) => {
                    let mut rule_warnings = check_singletons(&rule, &kb);
                    warnings.append(&mut rule_warnings);
                    let rule = rewrite_rule(rule, &mut kb);

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
        self.messages.extend(warnings.iter().map(|m| Message {
            kind: MessageKind::Warning,
            msg: m.to_owned(),
        }));

        Ok(())
    }

    // Used in integration tests
    pub fn load_str(&self, src: &str) -> PolarResult<()> {
        self.load(src, None)
    }

    /// Clear rules from the knowledge base
    pub fn clear_rules(&self) {
        let mut kb = self.kb.write().unwrap();
        kb.rules.clear();
        kb.sources = Sources::default();
        kb.inline_queries.clear();
        self.loaded_content.write().unwrap().clear();
        self.loaded_files.write().unwrap().clear();
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
            rewrite_term(term, &mut kb)
        };
        let query = Goal::Query { term: term.clone() };
        let vm =
            PolarVirtualMachine::new(self.kb.clone(), trace, vec![query], self.messages.clone());
        Ok(Query::new(vm, term))
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
}
