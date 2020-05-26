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

pub struct Query {
    vm: PolarVirtualMachine,
    done: bool,
}

impl Query {
    pub fn debug(&mut self, set: bool) {
        if set {
            self.vm.start_debug();
        } else {
            self.vm.stop_debug();
        }
    }

    pub fn debug_info(&self) -> crate::DebugInfo {
        self.vm.debug_info()
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

#[derive(Default)]
pub struct Polar {
    pub kb: KnowledgeBase,
}

impl Polar {
    pub fn new() -> Self {
        Self {
            kb: KnowledgeBase::new(),
        }
    }

    pub fn load_str(&mut self, src: &str) -> PolarResult<()> {
        let lines = parser::parse_lines(src)?;
        for line in lines {
            match line {
                parser::Line::Rule(rule) => {
                    let name = rule.name.clone();
                    let rewritten_rule = rewrite_rule(rule, &mut self.kb);
                    let generic_rule = self.kb.rules.entry(name.clone()).or_insert(GenericRule {
                        name,
                        rules: vec![],
                    });
                    generic_rule.rules.push(rewritten_rule);
                }
                parser::Line::Query(_term) => {
                    // currently not handled
                }
            }
        }
        Ok(())
    }

    pub fn new_query(&mut self, query_string: &str) -> PolarResult<Query> {
        let term = parser::parse_query(query_string)?;
        Ok(self.new_query_from_term(term))
    }

    pub fn new_query_from_term(&mut self, term: Term) -> Query {
        let query = Goal::Query {
            term: rewrite_term(term, &mut self.kb),
        };
        let vm = PolarVirtualMachine::new(self.kb.clone(), vec![query]);
        Query { vm, done: false }
    }

    // @TODO: Direct load_rules endpoint.

    pub fn query(&mut self, query: &mut Query) -> PolarResult<QueryEvent> {
        query.vm.run()
    }

    pub fn external_call_result(&mut self, query: &mut Query, call_id: u64, value: Option<Term>) {
        query.vm.external_call_result(call_id, value)
    }

    pub fn external_question_result(&mut self, query: &mut Query, call_id: u64, result: bool) {
        query.vm.external_question_result(call_id, result)
    }

    // @TODO: Get external_id call for returning external instances from python.
    pub fn get_external_id(&mut self, query: &mut Query) -> u64 {
        query.vm.new_id()
    }

    /// Turn this Polar instance into a new TUI instance and run it
    #[cfg(feature = "tui_")]
    pub fn into_tui(self) {
        let app = crate::cli::tui::App::new(self);
        crate::cli::tui::run(app).expect("error in CLI")
    }
}
