use super::types::*;
use super::vm::*;

use super::parser::{parse_file, parse_query};

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

// Query as an iterator returns `None` after the first time `Done` is seen
impl Iterator for Query {
    type Item = QueryEvent;

    fn next(&mut self) -> Option<QueryEvent> {
        if self.done {
            return None;
        }
        let event = self.vm.run();
        if let QueryEvent::Done = event {
            self.done = true;
        }
        Some(event)
    }
}

pub struct Polar {
    pub kb: KnowledgeBase,
}

impl Polar {
    pub fn new() -> Self {
        Self {
            kb: KnowledgeBase::new(),
        }
    }

    pub fn load_str(&mut self, src: &str) {
        // @TODO: Return Errors
        let rules = parse_file(src);
        for rule in rules {
            let generic_rule = self
                .kb
                .rules
                .entry(rule.name.clone())
                .or_insert(GenericRule {
                    name: rule.name.clone(),
                    rules: vec![],
                });
            generic_rule.rules.push(rule);
        }
    }

    pub fn new_query(&self, query_string: &str) -> Query {
        let pred = parse_query(query_string);
        self.new_query_from_predicate(pred)
    }

    pub fn new_query_from_predicate(&self, predicate: Predicate) -> Query {
        let query = Goal::Query {
            predicate: predicate.clone(),
        };
        let vm = PolarVirtualMachine::new(self.kb.clone(), vec![query]);
        Query { vm, done: false }
    }

    // @TODO: Direct load_rules endpoint.

    pub fn query(&mut self, query: &mut Query) -> QueryEvent {
        query.vm.run()
    }

    pub fn result(&mut self, query: &mut Query, call_id: i64, value: Term) {
        unimplemented!();
    }

    #[cfg(test)]
    pub fn test_result(&mut self, query: &mut Query, name: &Symbol, value: i64) {
        query.vm.push_goal(Goal::Result {
            name: name.clone(),
            value,
        });
        query.vm.push_goal(Goal::Result {
            name: name.clone(),
            value,
        })
    }

    #[cfg(test)]
    pub fn new_query_from_external(&self, name: Symbol) -> Query {
        let vm = PolarVirtualMachine::new(
            self.kb.clone(),
            vec![Goal::Bindings, Goal::TestExternal { name }],
        );
        Query { vm }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_results() {
        let mut polar = Polar::new();
        polar.load_str("foo(1);foo(2);");
        let mut query = polar.new_query("foo(a)");

        let mut results = vec![];
        loop {
            let event = polar.query(&mut query);
            match event {
                QueryEvent::Done => break,
                QueryEvent::TestExternal { .. } => panic!("no external call"),
                QueryEvent::Result { bindings } => {
                    results.push(bindings.get(&Symbol("a".to_string())).unwrap().clone());
                }
                _ => (),
            }
        }
        assert_eq!(
            results
                .iter()
                .map(|result| result.value.clone())
                .collect::<Vec<Value>>(),
            vec![Value::Integer(1), Value::Integer(2)]
        );
    }

    #[test]
    fn test_external() {
        let a = Symbol("a".to_string());
        let mut polar = Polar::new();
        let mut query = polar.new_query_from_external(a.clone());

        let mut results = vec![];
        loop {
            let event = polar.query(&mut query);
            match event {
                QueryEvent::Done => break,
                QueryEvent::TestExternal { name } => polar.test_result(&mut query, &name, 1),
                QueryEvent::Result { bindings } => {
                    results.push(bindings.get(&a).unwrap().clone());
                }
                _ => (),
            }
        }
        assert_eq!(
            results
                .iter()
                .map(|result| result.value.clone())
                .collect::<Vec<Value>>(),
            vec![Value::Integer(1)]
        );
    }
}
