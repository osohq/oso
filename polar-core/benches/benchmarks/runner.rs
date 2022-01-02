use std::sync::Arc;

use polar_core::{
    events::*, kb::Bindings, parser, polar::Polar, query::Query, source, sources::Source,
};

pub fn runner_from_query(q: &str) -> Runner {
    let polar = Polar::new();
    let query_term = parser::parse_query(&source!(q)).unwrap();
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

    pub fn next(&mut self) -> QueryEvent {
        self.query.next_event().expect("query errored")
    }

    pub fn run(&mut self) {
        loop {
            let event = self.next();
            match event {
                QueryEvent::Result { bindings, .. } => return self.handle_result(bindings),
                QueryEvent::Done { .. } if self.expected_result.is_some() => {
                    panic!("Result expected")
                }
                QueryEvent::Done { .. } => break,
                QueryEvent::Debug { message } => self.handle_debug(message),
                event => todo!("{:?}", event),
            }
        }
    }

    fn handle_result(&mut self, bindings: Bindings) {
        if let Some(ref expected_bindings) = self.expected_result {
            assert_eq!(expected_bindings, &bindings);
        }
    }

    fn handle_debug(&mut self, _: String) {}
}

impl std::ops::Deref for Runner {
    type Target = Polar;

    fn deref(&self) -> &Self::Target {
        &self.polar
    }
}
