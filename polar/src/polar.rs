use super::parser;
use super::types::*;

use std::collections::HashMap;
use std::f32::consts::E;

// Api for polar.
// Everything here has a corollary in lib that exposes it over ffi.

// Tracks the lifecycle of the query.
pub enum QueryState {
    New,
    ExternalCall,
    ReturnResult,
    Done,
}

pub struct Query {
    //query_string: String,
    predicate: Predicate,

    // WOW HACK
    done: bool,
    results: Vec<Environment>,
    results_returned: usize,
}

impl Query {
    // pub fn new_from_string(query_string: String) -> Self {
    //     let results = vec![
    //         Environment{ bindings: HashMap::new()}
    //     ];
    //
    //     Query {
    //         query_string,
    //
    //         results,
    //         results_returned: 0,
    //     }
    // }

    pub fn new_from_pred(predicate: Predicate) -> Self {
        let results = vec![
            Environment{ bindings: HashMap::new()}
        ];
        Query {
            predicate,
            results,
            done: false,
            results_returned: 0
        }
    }

}

pub struct Polar {
    pub knowledge_base: KnowledgeBase,
}

impl Polar {
    pub fn new() -> Self {
        let foo_rule = Rule {

            params: vec![
                Term {
                    id: 0,
                    value: Value::Symbol(Symbol("a".to_owned())),
                }
            ],
            body: vec![],
        };

        let generic_rule = GenericRule {
            name: "foo".to_owned(),
            rules: vec![foo_rule]
        };

        let mut generic_rules = HashMap::new();
        generic_rules.insert("foo".to_owned(), generic_rule);

        Self {
            knowledge_base: KnowledgeBase {
                types: HashMap::new(),
                rules: generic_rules,
            }
        }
    }

    // Takes in a string of polar syntax and adds it to the knowledge base.
    // Use when reading in a polar file.
    pub fn load_str(&mut self, src: String) {
        // @TODO: Return Errors
        let clauses = parser::parse_str(src).unwrap();
        for clause in clauses {
            //self.knowledge_base.push(clause)
        }
    }

    pub fn query(&mut self, query: &mut Query) -> QueryEvent {
        if query.done {
            return QueryEvent::Done
        }
        if let Some(generic_rule) = self.knowledge_base.rules.get(&query.predicate.name) {
            assert_eq!(generic_rule.name, query.predicate.name);
            let rule = &generic_rule.rules[0]; // just panic.
            let var = &rule.params[0]; // is a variable
            let val = &query.predicate.args[0]; // is a integer.
            assert!(matches!(val.value, Value::Integer(_)));
            if let Value::Symbol(s) = &var.value {
                let mut bindings = HashMap::new();
                bindings.insert(s.clone(), val.clone());
                let environment = Environment {
                    bindings
                };
                query.done = true;
                return QueryEvent::Result {environment}
            }
        }
        panic!("Make this return a result anyway");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let mut polar = Polar::new();
        let mut query = Query::new_from_pred(Predicate{ name: "foo".to_owned(), args: vec![Term{id: 2, value: Value::Integer(0)}]});
        let mut results = 0;
        loop {
            let event = polar.query(&mut query);
            match event {
                QueryEvent::Done => break,
                QueryEvent::Result {environment} => {
                    results += 1;
                    assert_eq!(environment.bindings[&Symbol("a".to_owned())].value, Term{id: 99, value: Value::Integer(0)}.value);
                }
            }
        }
        assert!(results == 1);
    }
}
