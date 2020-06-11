/// Polar benchmarking suite
use criterion::{criterion_group, criterion_main, Criterion};

use polar::{types::*, Polar, Query};

pub fn simple_queries(c: &mut Criterion) {
    let polar = Polar::new();
    let mut runner = Runner::default();
    c.bench_function("query 1=1", |b| {
        let query = polar::parser::parse_query("1 = 1").unwrap();
        b.iter(|| {
            let query = polar.new_query_from_term(query.clone());
            runner.run(query);
        })
    });
    c.bench_function("query 1=1,2=2", |b| {
        let query = polar::parser::parse_query("1 = 1, 2 = 2").unwrap();
        b.iter(|| {
            let query = polar.new_query_from_term(query.clone());
            runner.run(query);
        })
    });
}

/// Bench: create `target` rules of the form `f(i) := f(i-1)`
/// and measure the time to compute `f(target)`
/// This basically measures the performance of the rule sorting
pub fn too_many_predicates(c: &mut Criterion) {
    const target: usize = 10;
    let polar = Polar::new();
    polar.load_str("f(0);").unwrap();
    for i in 1..=target {
        polar
            .load_str(&format!("f({}) := f({});", i, i - 1))
            .unwrap();
    }
    let mut runner = Runner::default();
    runner.expected_result(Bindings::new());
    let query_term = polar::parser::parse_query(&format!("f({})", target)).unwrap();
    let query = polar.new_query_from_term(query_term.clone());
    runner.run(query);
    c.bench_function("10 queries to f", |b| {
        b.iter(|| {
            let query = polar.new_query_from_term(query_term.clone());
            runner.run(query);
        })
    });
}

criterion_group!(benches, simple_queries, too_many_predicates);
criterion_main!(benches);

/// Used to run benchmarks by providing helper methods
#[derive(Default)]
struct Runner {
    expected_result: Option<Bindings>,
}

impl Runner {
    fn expected_result(&mut self, bindings: Bindings) {
        self.expected_result = Some(bindings);
    }

    fn run(&mut self, query: Query) {
        for event in query {
            match event {
                Ok(QueryEvent::Result { bindings }) => return self.handle_result(bindings),
                Ok(QueryEvent::Done) if self.expected_result.is_some() => panic!("Result expected"),
                Ok(QueryEvent::Done) => return,
                Ok(_) => todo!(),
                Err(e) => panic!(e),
            }
        }
    }

    fn handle_result(&mut self, bindings: Bindings) {
        if let Some(ref expected_bindings) = self.expected_result {
            assert_eq!(expected_bindings, &bindings);
        }
    }
}
