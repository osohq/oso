use super::error::PolarResult;
use super::formatting::source_lines;
use super::parser;
use super::rewrites::*;
use super::types::*;
use super::vm::*;
use super::warnings::check_singletons;

use std::collections::{hash_map::Entry, HashMap, VecDeque};
use std::io::{stderr, Write};
use std::sync::{Arc, Mutex, RwLock};

pub struct Query {
    vm: PolarVirtualMachine,
    done: bool,
}

impl Query {
    pub fn next_event(&mut self) -> PolarResult<QueryEvent> {
        self.vm.run()
    }

    pub fn call_result(&mut self, call_id: u64, value: Option<Term>) -> PolarResult<()> {
        self.vm.external_call_result(call_id, value)
    }

    pub fn question_result(&mut self, call_id: u64, result: bool) {
        self.vm.external_question_result(call_id, result)
    }

    pub fn application_error(&mut self, message: String) {
        self.vm.external_error(message)
    }

    pub fn debug_command(&mut self, command: &str) -> PolarResult<()> {
        self.vm.debug_command(command)
    }
}

// Query as an iterator returns `None` after the first time `Done` is seen
impl Iterator for Query {
    type Item = PolarResult<QueryEvent>;

    fn next(&mut self) -> Option<PolarResult<QueryEvent>> {
        if self.done {
            return None;
        }
        let event = self.vm.run();
        if let Ok(QueryEvent::Done) = event {
            self.done = true;
        }
        Some(event)
    }
}

pub struct Polar {
    pub kb: Arc<RwLock<KnowledgeBase>>,
    pub messages: Arc<Mutex<VecDeque<Message>>>,
}

impl Polar {
    pub fn new() -> Self {
        Self {
            kb: Arc::new(RwLock::new(KnowledgeBase::new())),
            messages: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn get_message(&mut self) -> Option<Message> {
        if let Ok(mut messages) = self.messages.lock() {
            messages.pop_front()
        } else {
            None
        }
    }

    pub fn push_message(&self, kind: MessageKind, msg: String) {
        let mut messages = self.messages.lock().unwrap();
        messages.push_back(Message { kind, msg });
    }

    pub fn load_file(&self, src: &str, filename: Option<String>) -> PolarResult<()> {
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
                parser::Line::Rule(mut rule) => {
                    let mut rule_warnings = check_singletons(&rule, &kb);
                    warnings.append(&mut rule_warnings);
                    rewrite_rule(&mut rule, &mut kb);

                    let name = rule.name.clone();
                    let generic_rule = kb.rules.entry(name.clone()).or_insert(GenericRule {
                        name,
                        rules: vec![],
                    });
                    generic_rule.rules.push(Arc::new(rule));
                }
                parser::Line::Query(term) => {
                    kb.inline_queries.push(term);
                }
            }
        }
        let mut messages = self.messages.lock().unwrap();
        messages.extend(warnings.iter().map(|m| Message {
            kind: MessageKind::Warning,
            msg: m.to_owned(),
        }));

        Ok(())
    }

    // Used in integration tests
    pub fn load(&self, src: &str) -> PolarResult<()> {
        self.load_file(src, None)
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
            let mut term =
                parser::parse_query(src_id, src).map_err(|e| e.set_context(Some(&source), None))?;
            kb.sources.add_source(source, src_id);
            rewrite_term(&mut term, &mut kb);
            term
        };
        let query = Goal::Query { term };
        let vm =
            PolarVirtualMachine::new(self.kb.clone(), trace, vec![query], self.messages.clone());
        Ok(Query { done: false, vm })
    }

    pub fn new_query_from_term(&self, mut term: Term, trace: bool) -> Query {
        {
            let mut kb = self.kb.write().unwrap();
            rewrite_term(&mut term, &mut kb);
        }
        let query = Goal::Query { term };
        let vm =
            PolarVirtualMachine::new(self.kb.clone(), trace, vec![query], self.messages.clone());
        Query { done: false, vm }
    }

    // @TODO: Direct load_rules endpoint.

    pub fn get_external_id(&self) -> u64 {
        self.kb.read().unwrap().new_id()
    }

    pub fn register_constant(&mut self, name: Symbol, value: Term) {
        self.kb.write().unwrap().constant(name, value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_load_and_query() {
        let polar = Polar::new();
        let _query = polar.new_query("1 = 1", false);
        let _ = polar.load("f(_);");
    }
}
