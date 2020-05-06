use super::types::*;
use super::vm::*;

use std::collections::HashMap;
use std::f32::consts::E;
use std::rc::Rc;
use super::parser::{parse_query, parse_file};

// Api for polar.
// Everything here has a corollary in lib that exposes it over ffi.

pub struct Query {
    //query_string: String,
    predicate: Predicate,
    vm: PolarVirtualMachine,
}

type Match = Option<Env>;

pub struct Polar {
    pub kb: KnowledgeBase,
}

impl Polar {
    pub fn new() -> Self {
        // let foo_rule = Rule {
        //     name: "foo".to_owned(),
        //     params: vec![Term {
        //         id: 0,
        //         offset: 0,
        //         value: Value::Symbol(Symbol("a".to_owned())),
        //     }],
        //     body: vec![],
        // };
        //
        // let generic_rule = GenericRule {
        //     name: "foo".to_owned(),
        //     rules: vec![foo_rule],
        // };
        //
        // let mut generic_rules = HashMap::new();
        // generic_rules.insert("foo".to_owned(), generic_rule);

        Self {
            kb: KnowledgeBase::new()
        }
    }

    pub fn new_query(&self, query_string: &str) -> Query {
        let pred = parse_query(query_string);
        self.new_query_from_predicate(pred)
    }

    pub fn new_query_from_predicate(&self, predicate: Predicate) -> Query {
        let query = Instruction::Query(predicate.clone());
        let vm = PolarVirtualMachine::new(self.kb.clone(), vec![query]);
        Query {
            predicate,
            vm,
        }
    }

    pub fn load_str(&mut self, src: &str) {
        // @TODO: Return Errors
        let rules = parse_file(src);
        for rule in rules {
            let generic_rule = self.kb.rules.entry(rule.name.clone()).or_insert(GenericRule{name: rule.name.clone(), rules: vec![]});
            generic_rule.rules.push(rule);
        }
    }

    pub fn query(&mut self, query: &mut Query) -> QueryEvent {
        query.vm.run()
    }

    pub fn result(&mut self, query: &mut Query, result: i64) {
        query.vm.result(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let mut polar = Polar::new();
        let mut query = polar.new_query_from_predicate(Predicate {
            name: "foo".to_owned(),
            args: vec![Term {
                id: 2,
                offset: 0,
                value: Value::Integer(1),
            }],
        });

        /* The "external" loop. */
        let mut results = 0;
        loop {
            let event = polar.query(&mut query);
            match event {
                QueryEvent::Done => break,
                QueryEvent::External(_) => polar.result(&mut query, 1),
                QueryEvent::Result { bindings } => {
                    results += 1;
                    assert_eq!(
                        bindings[&Symbol("a".to_owned())].value,
                        Value::Integer(1)
                    );
                }
            }
        }
        assert_eq!(results, 1);
    }

    #[test]
    fn it_works_as_a_string() {
        let mut polar = Polar::new();
        let mut query = polar.new_query("foo(1)");

        /* The "external" loop. */
        let mut results = 0;
        loop {
            let event = polar.query(&mut query);
            match event {
                QueryEvent::Done => break,
                QueryEvent::External(_) => polar.result(&mut query, 1),
                QueryEvent::Result { bindings } => {
                    results += 1;
                    assert_eq!(
                        bindings[&Symbol("a".to_owned())].value,
                        Value::Integer(1)
                    );
                }
            }
        }
        assert_eq!(results, 1);
    }

    #[test]
    fn real_test() {
        let mut polar = Polar::new();
        polar.load_str("foo(1);foo(2);");
        let mut query = polar.new_query("foo(a)");

        /* The "external" loop. */
        let mut results = vec![];
        loop {
            let event = polar.query(&mut query);
            match event {
                QueryEvent::Done => break,
                QueryEvent::External(_) => panic!("No external call"),
                QueryEvent::Result { bindings } => {
                    results.push(bindings.get(&Symbol("a".to_string())).unwrap().clone());
                }
            }
        }
        assert_eq!(results.len(), 2);
    }
}
