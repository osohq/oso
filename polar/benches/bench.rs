//! Polar benchmarking suite

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

use std::iter::{once, repeat};

use polar::*;
use polar::{types::*, Polar, Query};

pub fn simple_queries(c: &mut Criterion) {
    let mut runner = Runner::new(Polar::new());
    c.bench_function("1=1", |b| {
        let query = polar::parser::parse_query("1 = 1").unwrap();
        b.iter(|| {
            let query = runner.new_query_from_term(query.clone());
            runner.run(query);
        })
    });
    c.bench_function("1=1,2=2", |b| {
        let query = polar::parser::parse_query("1 = 1, 2 = 2").unwrap();
        b.iter(|| {
            let query = runner.new_query_from_term(query.clone());
            runner.run(query);
        })
    });
}

/// Bench: create `TARGET` rules of the form `f(i) := f(i-1)`
/// and measure the time to compute `f(TARGET)`
/// This basically measures the performance of the rule sorting
pub fn too_many_predicates(c: &mut Criterion) {
    const TARGET: usize = 10;
    let polar = Polar::new();
    polar.load_str("f(0);").unwrap();
    for i in 1..=TARGET {
        polar
            .load_str(&format!("f({}) := f({});", i, i - 1))
            .unwrap();
    }
    let mut runner = Runner::new(polar);
    runner.expected_result(Bindings::new());
    let query_term = polar::parser::parse_query(&format!("f({})", TARGET)).unwrap();
    let query = runner.new_query_from_term(query_term.clone());
    runner.run(query);
    c.bench_function("f(N) := f(N-1) := ... := f(0)", |b| {
        b.iter(|| {
            let query = runner.new_query_from_term(query_term.clone());
            runner.run(query);
        })
    });
}

pub fn n_plus_one_queries(c: &mut Criterion) {
    /// Constructs `N` external results
    fn n_results(runner: &mut Runner, n: usize) {
        // make some instances. The literals dont change anything, but is convenient for
        // us to track things
        let child = runner.make_external(instance!("Person"));
        let grandchild = runner.make_external(instance!("Person"));

        let n_children = term!(vec![child; n + 1]); // n children in a list
        let one_grandchild = term!(vec![grandchild]);

        let grandchild_alice = vec![
            Some(one_grandchild.clone()),
            Some(term!("alice")),
            None,
            None,
        ];
        let grandchild_bert = vec![
            Some(one_grandchild.clone()),
            Some(term!("bert")),
            None,
            None,
        ];

        // List of n children (one term)
        // then n times grandchild -> name = Alice
        // then once grandchild -> name = Bert
        // then None (no more children)
        let external_calls = once(Some(n_children))
            .chain(repeat(grandchild_alice).take(n).flatten())
            .chain(grandchild_bert.into_iter())
            .chain(once(None))
            .collect();

        runner.external_calls(external_calls);
        runner.expected_result(Bindings::new());
    }

    let polar = Polar::new();
    polar
        .load_str(
            "
        has_grandchild_called(grandparent: Person, name) :=
            child in grandparent.children,
            grandchild in child.childern,
            grandchild.name = name;
    ",
        )
        .unwrap();
    let mut runner = Runner::new(polar);

    let n_array = [1, 5, 20];
    let delays = [0, 10_000, 100_000];

    let mut group = c.benchmark_group("N+1 query");
    for delay in &delays {
        for n in &n_array {
            n_results(&mut runner, *n);
            runner.external_cost = Some(std::time::Duration::new(0, *delay));
            group.throughput(Throughput::Elements(*n as u64));
            group.bench_function(
                BenchmarkId::from_parameter(format!("N={}, cost={}ns", n, delay)),
                |b| {
                    let query =
                        polar::parser::parse_query("has_grandchild_called(Person{}, \"bert\")")
                            .unwrap();
                    b.iter(|| {
                        let query = runner.new_query_from_term(query.clone());
                        runner.run(query);
                    })
                },
            );
        }
    }
    group.finish();
}

criterion_group!(
    benches,
    simple_queries,
    too_many_predicates,
    n_plus_one_queries
);
criterion_main!(benches);

/// Used to run benchmarks by providing helper methods
struct Runner {
    polar: Polar,
    expected_result: Option<Bindings>,
    external_calls: Vec<Option<Term>>,
    query: Option<Query>,
    external_cost: Option<std::time::Duration>,
}

impl Runner {
    fn new(polar: Polar) -> Self {
        Self {
            polar,
            expected_result: None,
            external_calls: Vec::new(),
            query: None,
            external_cost: None,
        }
    }

    fn expected_result(&mut self, bindings: Bindings) {
        self.expected_result = Some(bindings);
    }

    fn external_calls(&mut self, calls: Vec<Option<Term>>) {
        self.external_calls = calls;
        self.external_calls.reverse();
    }

    fn next(&mut self) -> QueryEvent {
        self.polar
            .query(self.query.as_mut().unwrap())
            .expect("query errored")
    }

    fn run(&mut self, query: Query) {
        self.query = Some(query);
        let mut external_calls = self.external_calls.clone();
        loop {
            let event = self.next();
            match event {
                QueryEvent::Result { bindings } => return self.handle_result(bindings),
                QueryEvent::Done if self.expected_result.is_some() => panic!("Result expected"),
                QueryEvent::Done => break,
                QueryEvent::MakeExternal { .. } => {}
                QueryEvent::ExternalIsa { call_id, .. } => self.handle_external_isa(call_id),
                QueryEvent::ExternalCall { call_id, .. } => {
                    let result = external_calls.pop().expect("a result ready to return");
                    self.handle_external_call(call_id, result);
                }
                QueryEvent::Debug { message } => self.handle_debug(message),
                event => todo!("{:?}", event),
            }
        }
        self.query = None;
    }

    fn handle_result(&mut self, bindings: Bindings) {
        if let Some(ref expected_bindings) = self.expected_result {
            assert_eq!(expected_bindings, &bindings);
        }
    }

    fn handle_external_isa(&mut self, call_id: u64) {
        self.polar
            .external_question_result(self.query.as_mut().unwrap(), call_id, true)
    }

    fn handle_external_call(&mut self, call_id: u64, result: Option<Term>) {
        if matches!(result, Some(Term { value: Value::ExternalInstance { .. }, ..}) | Some(Term { value: Value::List { .. }, ..}))
        {
            if let Some(cost) = self.external_cost {
                std::thread::sleep(cost);
            }
        }
        self.polar
            .external_call_result(self.query.as_mut().unwrap(), call_id, result)
            .unwrap();
    }

    fn handle_debug(&mut self, message: String) {
        // let mut repl = crate::cli::repl::Repl::new();
        // println!("{}", message);
        // let input = repl.plain_input("> ").unwrap();
        // self.polar
        //     .debug_command(self.query.as_mut().unwrap(), input)
        //     .unwrap();
    }

    fn make_external(&mut self, literal: InstanceLiteral) -> Term {
        let instance_id = self.polar.get_external_id();
        Term::new(Value::ExternalInstance(ExternalInstance {
            instance_id,
            literal: Some(literal),
        }))
    }
}

impl std::ops::Deref for Runner {
    type Target = Polar;

    fn deref(&self) -> &Self::Target {
        &self.polar
    }
}
