use super::error::PolarResult;
use super::formatting::source_lines;
use super::parser;
use super::rewrites::*;
use super::types::*;
use super::vm::*;

use std::collections::{hash_map::Entry, HashMap};
use std::io::{stderr, Write};
use std::sync::{Arc, RwLock};

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
    pub output: Arc<RwLock<Box<dyn Write>>>,
}

impl Polar {
    pub fn new(output: Option<Box<dyn Write>>) -> Self {
        Self {
            kb: Arc::new(RwLock::new(KnowledgeBase::new())),
            output: Arc::new(RwLock::new(output.unwrap_or_else(|| Box::new(stderr())))),
        }
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
        while let Some(line) = lines.pop() {
            match line {
                parser::Line::Rule(mut rule) => {
                    self.check_singletons(&rule, &kb);
                    rewrite_rule(&mut rule, &mut kb);

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

        Ok(())
    }

    /// Warn about singleton variables and unknown specializers in a rule,
    /// except those whose names start with `_`.
    pub fn check_singletons(&self, rule: &Rule, kb: &KnowledgeBase) {
        let mut singletons = HashMap::<Symbol, Option<Term>>::new();
        let mut check_term = |term: &Term| {
            if let Value::Variable(sym)
            | Value::RestVariable(sym)
            | Value::Pattern(Pattern::Instance(InstanceLiteral { tag: sym, .. })) = term.value()
            {
                if !sym.0.starts_with('_') && !kb.is_constant(sym) {
                    match singletons.entry(sym.clone()) {
                        Entry::Occupied(mut o) => {
                            o.insert(None);
                        }
                        Entry::Vacant(v) => {
                            v.insert(Some(term.clone()));
                        }
                    }
                }
            }
            term.clone()
        };

        for param in &rule.params {
            param.parameter.clone().map_replace(&mut check_term);
            if let Some(mut spec) = param.specializer.clone() {
                spec.map_replace(&mut check_term);
            }
        }
        rule.body.clone().map_replace(&mut check_term);

        let mut singletons = singletons
            .into_iter()
            .collect::<Vec<(Symbol, Option<Term>)>>();
        singletons.sort_by_key(|(_sym, term)| term.as_ref().map_or(0, |term| term.offset()));
        for (sym, singleton) in singletons {
            if let Some(term) = singleton {
                let mut writer = self.output.write().unwrap();
                let _ = if let Value::Pattern(..) = term.value() {
                    writeln!(&mut writer, "Unknown specializer {}", sym)
                } else {
                    writeln!(&mut writer, "Singleton variable {} is unused or undefined, see <https://docs.oso.dev/using/polar-syntax.html#variables>", sym)
                };
                if let Some(ref source) = kb.sources.get_source(&term) {
                    let _ = writeln!(&mut writer, "{}", source_lines(source, term.offset(), 0));
                }
            }
        }
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
        let vm = PolarVirtualMachine::new(
            self.kb.clone(),
            trace,
            vec![query],
            Some(self.output.clone()),
        );
        Ok(Query { done: false, vm })
    }

    pub fn new_query_from_term(&self, mut term: Term, trace: bool) -> Query {
        {
            let mut kb = self.kb.write().unwrap();
            rewrite_term(&mut term, &mut kb);
        }
        let query = Goal::Query { term };
        let vm = PolarVirtualMachine::new(
            self.kb.clone(),
            trace,
            vec![query],
            Some(self.output.clone()),
        );
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
        let polar = Polar::new(None);
        let _query = polar.new_query("1 = 1", false);
        let _ = polar.load("f(_);");
    }
}
