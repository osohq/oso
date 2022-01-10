use std::collections::HashMap;

use polar_core::parser;
use polar_core::terms::{Symbol, Value};
use polar_core::{polar::Polar, query::Query};

pub type Bindings = HashMap<Symbol, Value>;

pub fn runner_from_query(q: &str) -> Runner {
    let polar = Polar::new();
    let query_term = parser::parse_query(0, q).unwrap();
    let query = polar.new_query_from_term(query_term, false);
    Runner::new(polar, query)
}

/// Used to run benchmarks by providing helper methods
pub struct Runner {
    polar: Polar,
    expected_result: Option<Bindings>,
    query: Query,
}

impl Runner {
    pub fn new(polar: Polar, query: Query) -> Self {
        Self {
            polar,
            expected_result: None,
            query,
        }
    }
    pub fn expected_result(&mut self, bindings: Bindings) {
        self.expected_result = Some(bindings);
    }

    pub fn run(&mut self) {
        let Self {
            expected_result,
            query,
            ..
        } = self;
        if let Some(result) = query.run().next() {
            if let Some(expected) = expected_result.as_ref() {
                assert_eq!(expected, &result);
            }
        } else if expected_result.is_some() {
            panic!("Result expected")
        }
    }
}

impl std::ops::Deref for Runner {
    type Target = Polar;

    fn deref(&self) -> &Self::Target {
        &self.polar
    }
}
