use super::parser;
use super::types::*;

use std::collections::HashMap;
use std::f32::consts::E;
use std::rc::Rc;
use crate::parser::{parse_predicate, parse_query};

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
}

type Match = Option<Env>;

fn unify(left: &Term, right: &Term, env: &Env) -> Match {
    // TODO make parent environment and make env not mut
    let new_env = Environment::new(env);
    unify_inner(&left, &right, new_env).map(Rc::new)
}

fn unify_inner(left: &Term, right: &Term, env: Environment) -> Option<Environment> {
    match (&left.value, &right.value) {
        (Value::Symbol(_), _) => unify_var(left, right, env),
        (_, Value::Symbol(_)) => unify_var(right, left, env),
        (Value::List(left), Value::List(right)) => {
            if left.len() != right.len() {
                return None;
            }

            let mut env = env;
            for (left, right) in left.iter().zip(right) {
                let maybe_match = unify_inner(left, &right, env);
                if let Some(match_) = maybe_match {
                    env = match_;
                } else {
                    return None;
                }
            }

            Some(env)
        }
        (Value::Integer(left), Value::Integer(right)) => {
            if left == right {
                Some(env)
            } else {
                None
            }
        }
        // TODO other cases
        (_, _) => unimplemented!(),
    }
}

fn unify_var(left: &Term, right: &Term, mut env: Environment) -> Option<Environment> {
    let left_sym = if let Value::Symbol(left_sym) = &left.value {
        left_sym
    } else {
        panic!("unify_var must be called with left as a Symbol.");
    };

    if let Some(left_value) = env.get(&left_sym) {
        return unify_inner(&left_value.clone(), right, env);
    }

    if let Value::Symbol(right_sym) = &right.value {
        if let Some(right_value) = env.get(&right_sym) {
            return unify_inner(left, &right_value.clone(), env);
        }
    }

    env.set(left_sym.clone(), right.clone());
    return Some(env);
}

impl Query {
    pub fn new_from_string(query_string: String) -> Self {
        let predicate = parse_query(query_string).unwrap(); // @TODO: Errors.
        let results = vec![Environment::empty()];
        Query {
            predicate,
            done: false,
        }
    }

    pub fn new_from_pred(predicate: Predicate) -> Self {
        let results = vec![Environment::empty()];
        Query {
            predicate,
            done: false,
        }
    }
}

pub struct Polar {
    pub knowledge_base: KnowledgeBase,
}

impl Polar {
    pub fn new() -> Self {
        let foo_rule = Rule {
            params: vec![Term {
                id: 0,
                value: Value::Symbol(Symbol("a".to_owned())),
            }],
            body: vec![],
        };

        let generic_rule = GenericRule {
            name: "foo".to_owned(),
            rules: vec![foo_rule],
        };

        let mut generic_rules = HashMap::new();
        generic_rules.insert("foo".to_owned(), generic_rule);

        Self {
            knowledge_base: KnowledgeBase {
                types: HashMap::new(),
                rules: generic_rules,
            },
        }
    }

    // Takes in a string of polar syntax and adds it to the knowledge base.
    // Use when reading in a polar file.
    pub fn load_str(&mut self, src: String) {
        // @TODO: Return Errors
        let clauses = parser::parse_source(src).unwrap();
        for clause in clauses {
            //self.knowledge_base.push(clause)
        }
    }

    pub fn query(&mut self, query: &mut Query) -> QueryEvent {
        if query.done {
            return QueryEvent::Done;
        }

        if let Some(generic_rule) = self.knowledge_base.rules.get(&query.predicate.name) {
            assert_eq!(generic_rule.name, query.predicate.name);
            let rule = &generic_rule.rules[0]; // just panic.
            let var = &rule.params[0]; // is a variable
            let val = &query.predicate.args[0]; // is a integer.

            let env = Rc::new(Environment::empty());

            let matched = unify(var, val, &env);
            if let Some(match_) = matched {
                assert!(matches!(val.value, Value::Integer(_)));
                query.done = true;
                return QueryEvent::Result {
                    bindings: match_.flatten_bindings(),
                };
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

        let mut queries = vec![];
        queries.push(Query::new_from_pred(Predicate {
            name: "foo".to_owned(),
            args: vec![Term {
                id: 2,
                value: Value::Integer(0),
            }],
        }));
        queries.push(Query::new_from_string("foo(0)".to_owned()));

        for mut query in &mut queries {
            let mut results = 0;
            loop {
                let event = polar.query(&mut query);
                match event {
                    QueryEvent::Done => break,
                    QueryEvent::Result { bindings } => {
                        results += 1;
                        assert_eq!(
                            bindings[&Symbol("a".to_owned())].value,
                            Value::Integer(0)
                        );
                    }
                }
            }
            assert_eq!(results, 1);
        }
    }
}
