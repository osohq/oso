use super::types::*;
use super::vm::*;

use super::parser::{parse_file, parse_query};

// Api for polar.
// Everything here has a corollary in lib that exposes it over ffi.

pub struct Query {
    //query_string: String,
    //predicate: Predicate,
    vm: PolarVirtualMachine,
}

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
            kb: KnowledgeBase::new(),
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
        Query { vm }
    }

    #[cfg(test)]
    pub fn new_query_from_external(&self, name: Symbol) -> Query {
        let vm = PolarVirtualMachine::new(
            self.kb.clone(),
            vec![Goal::Bindings, Goal::External { name }],
        );
        Query { vm }
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

    pub fn query(&mut self, query: &mut Query) -> QueryEvent {
        query.vm.run()
    }

    pub fn result(&mut self, query: &mut Query, name: &Symbol, value: i64) {
        query.vm.push_goal(Goal::Result{name: name.clone(), value})
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
                QueryEvent::External { .. } => panic!("no external call"),
                QueryEvent::Result { bindings } => {
                    results.push(bindings.get(&Symbol("a".to_string())).unwrap().clone());
                }
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
                QueryEvent::External { name } => polar.result(&mut query, &name, 1),
                QueryEvent::Result { bindings } => {
                    results.push(bindings.get(&a).unwrap().clone());
                }
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

    #[test]
    fn test_debugger() {
        use std::collections::HashMap;
        let g = Goal::Backtrack;
        let mut v = vec![];
        v.push(1);
        v.push(2);
        v.push(3);
        let mut h = HashMap::new();
        h.insert(1, 1);
        h.insert(2, 2);
        h.insert(3, 3);        
    }
}
