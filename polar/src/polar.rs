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
    pub kb: KnowledgeBase,
}

impl Polar {
    pub fn new() -> Self {
        Self {
            kb: KnowledgeBase::new(),
        }
    }

    pub fn load_str(&mut self, src: &str) -> PolarResult<()> {
        let rules = parse_file(src)?;
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
        Ok(())
    }

    pub fn new_query(&self, query_string: &str) -> PolarResult<Query> {
        let pred = parse_query(query_string)?;
        Ok(self.new_query_from_predicate(pred))
    }

    pub fn new_query_from_predicate(&self, predicate: Predicate) -> Query {
        let query = Goal::Query {
            predicate: predicate.clone(),
        };
        let vm = PolarVirtualMachine::new(self.kb.clone(), vec![query]);
        Query { vm, done: false }
    }

    // @TODO: Direct load_rules endpoint.

    pub fn query(&mut self, query: &mut Query) -> PolarResult<QueryEvent> {
        query.vm.run()
    }

    pub fn result(&mut self, _query: &mut Query, _call_id: i64, _value: Term) -> PolarResult<()> {
        Err(PolarError::Unimplemented("result".to_string()))
    }

    #[cfg(test)]
    pub fn test_result(&mut self, query: &mut Query, name: &Symbol, value: Option<i64>) {
        query.vm.push_goal(Goal::Result {
            name: name.clone(),
            value,
        });
    }

    #[cfg(test)]
    pub fn new_query_from_external(&self, name: Symbol) -> Query {
        let vm = PolarVirtualMachine::new(self.kb.clone(), vec![Goal::TestExternal { name }]);
        Query { vm, done: false }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::iter::FromIterator;

    use super::*;
    use permute::permute;

    fn result_values(results: Vec<Term>) -> Vec<Value> {
        results.into_iter().map(|t| t.value).collect()
    }

    fn qvar(results: &Vec<HashMap<Symbol, Value>>, var: &str) -> Vec<Value> {
        results
            .iter()
            .map(|bindings| bindings.get(&Symbol(var.to_string())).unwrap().clone())
            .collect()
    }

    fn query_results(polar: &mut Polar, mut query: Query) -> Vec<HashMap<Symbol, Value>> {
        let mut results = vec![];
        loop {
            let event = polar.query(&mut query).unwrap();
            match event {
                QueryEvent::Done => break,
                QueryEvent::TestExternal { .. } => panic!("no external call"),
                QueryEvent::Result { bindings } => {
                    results.push(bindings.into_iter().map(|(k, v)| (k, v.value)).collect());
                }
                _ => panic!("unexpected event"),
            }
        }

        results
    }

    /// Adapted from <http://web.cse.ohio-state.edu/~stiff.4/cse3521/prolog-resolution.html>
    #[test]
    fn test_functions() {
        let mut polar = Polar::new();
        polar.load_str("f(1); f(2); g(1); g(2); h(2); k(x) := f(x), h(x), g(x);");

        let query = polar.new_query("k(1)");
        assert_eq!(query_results(&mut polar, query).len(), 0);

        let query = polar.new_query("k(2)");
        assert_eq!(query_results(&mut polar, query).len(), 1);

        let query = polar.new_query("k(3)");
        assert_eq!(query_results(&mut polar, query).len(), 0);

        let query = polar.new_query("k(a)");
        assert_eq!(
            qvar(&query_results(&mut polar, query), "a"),
            vec![value!(2)]
        );
    }

    /// Adapted from <http://web.cse.ohio-state.edu/~stiff.4/cse3521/prolog-resolution.html>
    #[test]
    fn test_jealous() {
        let mut polar = Polar::new();
        polar.load_str(
            r#"loves("vincent", "mia");
               loves("marcellus", "mia");
               jealous(a, b) := loves(a, c), loves(b, c);"#,
        );

        let query = polar.new_query("jealous(who, of)");
        let results = query_results(&mut polar, query);
        let jealous = |who: &str, of: &str| {
            assert!(
                &results.contains(&HashMap::from_iter(vec![
                    (sym!("who"), value!(who)),
                    (sym!("of"), value!(of))
                ])),
                "{} is not jealous of {} (but should be)",
                who,
                of
            );
        };
        assert_eq!(results.len(), 4);
        jealous("vincent", "vincent");
        jealous("vincent", "marcellus");
        jealous("marcellus", "vincent");
        jealous("marcellus", "marcellus");
    }

    #[test]
    fn test_nested_rule() {
        let mut polar = Polar::new();
        polar.load_str("f(x) := g(x); g(x) := h(x); h(2); g(x) := j(x); j(4);");

        let query = polar.new_query("f(2)");
        assert_eq!(query_results(&mut polar, query), vec![HashMap::new()]);

        let query = polar.new_query("f(3)");
        assert!(query_results(&mut polar, query).is_empty());

        let query = polar.new_query("f(4)");
        assert_eq!(query_results(&mut polar, query), vec![HashMap::new()]);

        let query = polar.new_query("j(4)");
        assert_eq!(query_results(&mut polar, query), vec![HashMap::new()]);
    }

    #[test]
    /// A functions permutation that is known to fail.
    fn test_bad_functions() {
        let mut polar = Polar::new();
        polar.load_str("f(2); f(1); g(1); g(2); h(2); k(x) := f(x), h(x), g(x);");
        let query = polar.new_query("k(a)");
        assert_eq!(
            qvar(&query_results(&mut polar, query), "a"),
            vec![value!(2)]
        );
    }

    #[test]
    fn test_functions_reorder() {
        // TODO (dhatch): Reorder f(x), h(x), g(x)
        let parts = vec![
            "f(1)",
            "f(2)",
            "g(1)",
            "g(2)",
            "h(2)",
            "k(x) := f(x), h(x), g(x)",
        ];

        let mut i = 0;
        for permutation in permute(parts) {
            let mut polar = Polar::new();

            let mut joined = permutation.join(";");
            joined.push(';');
            polar.load_str(&joined);

            let query = polar.new_query("k(1)");
            assert_eq!(
                query_results(&mut polar, query).len(),
                0,
                "k(1) failed for permutation {:?}",
                &permutation
            );

            let query = polar.new_query("k(2)");
            assert_eq!(
                query_results(&mut polar, query).len(),
                1,
                "k(2) failed for permutation {:?}",
                &permutation
            );

            let query = polar.new_query("k(a)");
            assert_eq!(
                qvar(&query_results(&mut polar, query), "a"),
                vec![value!(2)],
                "k(a) failed for permutation {:?}",
                &permutation
            );

            i += 1;
            println!("permute: {}", i);
        }
    }

    #[test]
    fn test_results() {
        let mut polar = Polar::new();
        polar.load_str("foo(1); foo(2);");
        let query = polar.new_query("foo(a)");
        assert_eq!(
            qvar(&query_results(&mut polar, query), "a"),
            vec![value!(1), value!(2)]
        );
    }

    #[test]
    /// From Aït-Kaci's WAM tutorial (1999), page 34.
    fn test_ait_kaci_34() {
        let mut polar = Polar::new();
        polar.load_str(
            r#"a() := b(x), c(x);
               b(x) := e(x);
               c(1);
               e(x) := f(x);
               e(x) := g(x);
               f(2);
               g(1);"#,
        );

        let query = polar.new_query("a()");
        let results = query_results(&mut polar, query);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_external() {
        let a = Symbol("a".to_string());
        let mut polar = Polar::new();
        let mut query = polar.new_query_from_external(a.clone());

        let mut externals = vec![1, 2, 3];
        let mut results = vec![];
        loop {
            let event = polar.query(&mut query).unwrap();
            match event {
                QueryEvent::Done => break,
                QueryEvent::TestExternal { name } => {
                    polar.test_result(&mut query, &name, externals.pop())
                }
                QueryEvent::Result { bindings } => {
                    results.push(bindings.get(&a).unwrap().clone());
                }
                _ => (),
            }
        }
        assert_eq!(result_values(results), vec![value!(1)]);
    }
}
