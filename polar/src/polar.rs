use super::rewrites::*;
use super::types::*;
use super::vm::*;

use super::parser;

// @TODO: This should probably go in the readme, it's meant to be the things you'd have to know to add
// new language bindings.

// This is the interface between the polar library (rust) and the application language (python).
// This interface uses rust types to make it easy to write tests against, see "lib.rs" for the ffi
// translation layer that exposes the library over a c compatable interface for python and other
// languages to call.
// The library is compiled as a static library which can be easily linked into a python module.
// The build step produces a "polar.h" file which is the interface needed to call into it.
// That polar.h file is generated from the functions and types exposed in lib.rs.

// The general usage of this library by an application language is like this.
// Call polar_new to create a new polar instance. All the state is contained in this type (or other
// types linked to it). There is no global state (except in some ffi details) so you can have multiple
// instances of polar and it's not a problem.

// With an Instance you can call polar_load_str() to load some polar code into the knowledge base.
// With an Instance you can call polar_new_query() or polar_new_query_from_predicate() to create a
// query object that can be used to execute a query against the knowledge base.

// The execution of a query is based around an event loop which enables the polar library to return
// control back to the application when something happens that requires interop with the application.
// There are events for external calls and for yielding results.
// Running a query looks something like this.

// polar = polar_new();
// polar_load_str(polar, "foo(1);foo(2);");
// query = polar_new_query(polar, "foo(x)");
// event = polar_query(query);
// while event != Event::Done {
//     if event == Event::Result(bindings) {
//         yield event.bindings // or collect them up or something
//     } else if event == Event::External(instance_info) {
//         result = python_call_external(instance_info)
//         if result {
//           polar_result(instance_info, result);
//         } else {
//           polar_result(instance_info, None);
//         }
//     }
//     event = polar_query(query);
// }

// When external calls are requested they have an associated id. You will typically get multiple external
// call events and you can return an event each time until you don't have anymore. When you are out
// or if you didn't have any to begin with you call polar_result with a null value.
// This polling for the results enables hooking the event loop up to generators or other poll based
// machinery in the application language.

// @TODO: Once the external constructor stuff and instance ids are worked out explain them.

use std::sync::{Arc, RwLock};

pub struct Query {
    vm: PolarVirtualMachine,
    done: bool,
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

#[derive(Default)]
pub struct Load {
    lines: Vec<parser::Line>,
    src_id: u64,
}

#[derive(Clone, Default)]
pub struct Polar {
    pub kb: Arc<RwLock<KnowledgeBase>>,
}

impl Polar {
    pub fn new() -> Self {
        Self {
            kb: Arc::new(RwLock::new(KnowledgeBase::new())),
        }
    }

    pub fn new_load(&self, src: &str) -> PolarResult<Load> {
        let mut lines = parser::parse_lines(src)?;
        lines.reverse();
        let mut kb = self.kb.write().unwrap();
        let src_id = kb.new_id();
        kb.sources.add_source(
            Source {
                filename: None,
                src: src.to_owned(),
            },
            src_id,
        );
        Ok(Load { lines, src_id })
    }

    pub fn load(&self, load: &mut Load) -> PolarResult<Option<Query>> {
        while let Some(line) = load.lines.pop() {
            match line {
                parser::Line::Rule(mut rule) => {
                    let name = rule.name.clone();
                    let mut kb = self.kb.write().unwrap();
                    rewrite_rule(&mut rule, &mut kb, load.src_id);
                    let generic_rule = kb.rules.entry(name.clone()).or_insert(GenericRule {
                        name,
                        rules: vec![],
                    });
                    generic_rule.rules.push(rule);
                }
                parser::Line::Query(term) => {
                    return Ok(Some(self.new_query_from_term(term)));
                }
            }
        }

        Ok(None)
    }

    pub fn load_str(&self, src: &str) -> PolarResult<()> {
        let mut load = self.new_load(src)?;
        while let Some(_query) = self.load(&mut load)? {
            // Queries are ignored in `load_str`.
            continue;
        }

        Ok(())
    }

    pub fn new_query(&self, src: &str) -> PolarResult<Query> {
        let mut term = parser::parse_query(src)?;
        let source = Source {
            src: src.to_string(),
            filename: None,
        };
        {
            let mut kb = self.kb.write().unwrap();
            let src_id = kb.new_id();
            kb.sources.add_source(source, src_id);
            rewrite_term(&mut term, &mut kb, src_id);
        }
        let query = Goal::Query { term };
        let vm = PolarVirtualMachine::new(self.kb.clone(), vec![query]);
        Ok(Query { done: false, vm })
    }

    // TODO(gj): Ensure we always pass the source along with the parsed Term for debugging / error
    // handling purposes.
    pub fn new_query_from_term(&self, mut term: Term) -> Query {
        {
            let mut kb = self.kb.write().unwrap();
            rewrite_term(&mut term, &mut kb, 0);
        }
        let query = Goal::Query { term };
        let vm = PolarVirtualMachine::new(self.kb.clone(), vec![query]);
        Query { done: false, vm }
    }

    #[cfg(not(feature = "repl"))]
    pub fn new_query_from_repl(&self) -> PolarResult<Query> {
        Err(PolarError::Runtime(RuntimeError::Unsupported {
            msg: "The REPL is not supported in this build.".to_string(),
        }))
    }

    #[cfg(feature = "repl")]
    pub fn new_query_from_repl(&self) -> PolarResult<Query> {
        let mut repl = crate::cli::repl::Repl::new();
        let s = repl.input("Enter query:");
        match s {
            Ok(s) => self.new_query(&s),
            Err(_) => Err(PolarError::Operational(OperationalError::Unknown)),
        }
    }

    // @TODO: Direct load_rules endpoint.

    pub fn query(&self, query: &mut Query) -> PolarResult<QueryEvent> {
        query.vm.run()
    }

    pub fn external_call_result(
        &self,
        query: &mut Query,
        call_id: u64,
        value: Option<Term>,
    ) -> PolarResult<()> {
        query.vm.external_call_result(call_id, value)
    }

    pub fn debug_command(&self, query: &mut Query, command: String) -> PolarResult<()> {
        query.vm.debug_command(&command)
    }

    pub fn external_question_result(&self, query: &mut Query, call_id: u64, result: bool) {
        query.vm.external_question_result(call_id, result)
    }

    // @TODO: Get external_id call for returning external instances from python.
    pub fn get_external_id(&self) -> u64 {
        self.kb.read().unwrap().new_id()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]

    fn can_load_and_query() {
        let polar = Polar::new();
        let _query = polar.new_query("1 = 1");
        let _ = polar.load_str("f(x);");
    }
}
