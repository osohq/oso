//! Polar benchmarking suite

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

use std::iter::{once, repeat};

use polar_core::*;
use polar_core::{polar::Polar, polar::Query, types::*};

fn runner_from_query(q: &str) -> Runner {
    let polar = Polar::new();
    let query_term = parser::parse_query(0, q).unwrap();
    let query = polar.new_query_from_term(query_term, false);
    Runner::new(polar, query)
}

pub fn simple_queries(c: &mut Criterion) {
    c.bench_function("unify_once", |b| {
        b.iter_batched(
            || runner_from_query("1=1"),
            |mut runner| runner.run(),
            criterion::BatchSize::SmallInput,
        )
    });
    c.bench_function("unify_twice", |b| {
        b.iter_batched(
            || runner_from_query("1=1 and 2=2"),
            |mut runner| runner.run(),
            criterion::BatchSize::SmallInput,
        )
    });
}

pub fn fib(c: &mut Criterion) {
    let policy = "
        fib(0, 1) if cut;
        fib(1, 1) if cut;
        fib(n, a+b) if fib(n-1, a) and fib(n-2, b);
    ";

    let n_array = [
        5i64, // 10, 15, 20,
    ];

    fn fib(n: i64) -> i64 {
        match n {
            0 => 1,
            1 => 1,
            n => fib(n - 1) + fib(n - 2),
        }
    }

    let mut group = c.benchmark_group("fib");
    for n in &n_array {
        group.bench_function(BenchmarkId::from_parameter(format!("{}", n)), |b| {
            b.iter_batched(
                || {
                    let mut runner = runner_from_query(&format!("fib({}, result)", n));
                    runner.load_str(policy).unwrap();
                    runner.expected_result(maplit::hashmap!(
                        sym!("result") => term!(fib(*n))
                    ));
                    runner
                },
                |mut runner| {
                    runner.run();
                },
                criterion::BatchSize::SmallInput,
            )
        });
    }

    group.finish();
}

pub fn prime(c: &mut Criterion) {
    let policy = "
        prime(x) if x in [
            2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97
        ];
    ";

    fn prime(n: &u8) -> bool {
        let small_primes = [
            2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83,
            89, 97,
        ];
        small_primes.iter().any(|m| *m == *n)
    }

    let mut group = c.benchmark_group("prime");
    for n in &[3, 23, 43, 83, 255] {
        group.bench_function(BenchmarkId::from_parameter(format!("{}", n)), |b| {
            b.iter_batched(
                || {
                    let mut runner = runner_from_query(&format!("prime({})", n));
                    runner.load_str(policy).unwrap();
                    if prime(n) {
                        runner.expected_result(maplit::hashmap!())
                    }
                    runner
                },
                |mut runner| {
                    runner.run();
                },
                criterion::BatchSize::SmallInput,
            )
        });
    }

    group.finish();
}

/// Bench: create `TARGET` rules of the form `f(i) if f(i-1)`
/// and measure the time to compute `f(TARGET)`
/// This basically measures the performance of the rule sorting
pub fn many_rules(c: &mut Criterion) {
    const TARGET: usize = 10;
    fn make_runner() -> Runner {
        let mut runner = runner_from_query(&format!("f({})", TARGET));
        runner.load_str("f(0);").unwrap();
        for i in 1..=TARGET {
            runner
                .load_str(&format!("f({}) if f({});", i, i - 1))
                .unwrap();
        }
        runner.expected_result(Bindings::new());
        runner
    }

    c.bench_function("many_rules", |b| {
        b.iter_batched(
            make_runner,
            |mut runner| runner.run(),
            criterion::BatchSize::SmallInput,
        )
    });
}

/// Bench: Example policy showing N+1 query behaviour.
/// The first query is to `grandparent.children`, then
/// for every result `child`, there will be a further query to
/// `child.children`. Implemented naively, this results in N+1
/// database lookups.
pub fn n_plus_one_queries(c: &mut Criterion) {
    let policy = "
        has_grandchild_called(grandparent: Person, name) if
            child in grandparent.children and
            grandchild in child.childern and
            grandchild.name = name;
    ";

    /// Constructs `N` external results
    fn n_results(runner: &mut Runner, n: usize) {
        runner.register_pseudo_class("Person");

        // Make some instances. The literals dont change anything, but is convenient for
        // us to track things.
        let child = runner.make_external(instance!("Person"));
        let grandchild = runner.make_external(instance!("Person"));

        let n_children = term!(vec![child; n]); // n children in a list
        let one_grandchild = term!(vec![grandchild]);

        let grandchild_alice = vec![
            Some(one_grandchild.clone()),
            Some(term!("alice")),
            None,
            None,
        ];
        let grandchild_bert = vec![Some(one_grandchild), Some(term!("bert")), None, None];

        // List of n children (one term)
        // then n-1 times grandchild -> name = Alice
        // then once grandchild -> name = Bert
        // then None (no more children)
        let external_calls = once(Some(n_children))
            .chain(repeat(grandchild_alice).take(n - 1).flatten())
            .chain(grandchild_bert.into_iter())
            .chain(once(None))
            .collect();

        runner.external_calls(external_calls);
        runner.expected_result(Bindings::new());
    }

    let n_array = [1, 5];
    let delays = [10_000];

    let mut group = c.benchmark_group("n_plus_one");
    for delay in &delays {
        for n in &n_array {
            group.bench_function(
                BenchmarkId::from_parameter(format!("{}, cost={}ns", n, delay)),
                |b| {
                    b.iter_batched(
                        || {
                            let mut runner =
                                runner_from_query("has_grandchild_called(new Person{}, \"bert\")");
                            runner.register_pseudo_class("Person");
                            runner.load_str(policy).unwrap();
                            n_results(&mut runner, *n);
                            runner.external_cost = Some(std::time::Duration::new(0, *delay));
                            runner
                        },
                        |mut runner| {
                            runner.run();
                            // check: we do actually run N+1 queries
                            assert_eq!(runner.calls_count, 1 + *n);
                        },
                        criterion::BatchSize::SmallInput,
                    )
                },
            );
        }
    }
    group.finish();
}

criterion_group!(
    benches,
    simple_queries,
    many_rules,
    n_plus_one_queries,
    fib,
    prime
);
criterion_main!(benches);

/// Used to run benchmarks by providing helper methods
struct Runner {
    polar: Polar,
    expected_result: Option<Bindings>,
    external_calls: Vec<Option<Term>>,
    query: Query,
    external_cost: Option<std::time::Duration>,
    calls_count: usize,
}

impl Runner {
    fn new(polar: Polar, query: Query) -> Self {
        Self {
            polar,
            expected_result: None,
            external_calls: Vec::new(),
            query,
            external_cost: None,
            calls_count: 0,
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
        self.query.next_event().expect("query errored")
    }

    fn run(&mut self) {
        loop {
            let event = self.next();
            match event {
                QueryEvent::Result { bindings, .. } => return self.handle_result(bindings),
                QueryEvent::Done if self.expected_result.is_some() => panic!("Result expected"),
                QueryEvent::Done => break,
                QueryEvent::MakeExternal { .. } => {}
                QueryEvent::ExternalIsa { call_id, .. } => self.handle_external_isa(call_id),
                QueryEvent::ExternalCall { call_id, .. } => {
                    self.handle_external_call(call_id);
                }
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

    fn handle_external_isa(&mut self, call_id: u64) {
        self.query.question_result(call_id, true)
    }

    fn handle_external_call(&mut self, call_id: u64) {
        let result = self.external_calls.pop().expect("more results");
        if matches!(result.as_ref().map(Term::value), Some(Value::ExternalInstance { .. }) | Some(Value::List { .. }))
        {
            self.calls_count += 1;
            if let Some(cost) = self.external_cost {
                std::thread::sleep(cost);
            }
        }
        self.query.call_result(call_id, result).unwrap();
    }

    fn handle_debug(&mut self, _: String) {}

    fn make_external(&mut self, literal: InstanceLiteral) -> Term {
        let instance_id = self.polar.get_external_id();
        Term::new_from_test(Value::ExternalInstance(ExternalInstance {
            instance_id,
            constructor: Some(Term::new_from_test(Value::InstanceLiteral(literal))),
            repr: None,
        }))
    }

    pub fn register_pseudo_class(&mut self, name: &str) {
        self.polar
            .register_constant(Symbol::new(name), Term::new_temporary(Value::Boolean(true)));
    }
}

impl std::ops::Deref for Runner {
    type Target = Polar;

    fn deref(&self) -> &Self::Target {
        &self.polar
    }
}
