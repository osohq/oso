use criterion::{criterion_group, BenchmarkId, Criterion};
use std::fmt::Write;

use polar_core::*;
use polar_core::{kb::Bindings, polar::Polar, terms::*};

use super::runner::{runner_from_query, Runner};

pub fn simple_queries(c: &mut Criterion) {
    c.bench_function("unify_once", |b| {
        b.iter_batched_ref(
            || runner_from_query("1=1"),
            |runner| runner.run(),
            criterion::BatchSize::SmallInput,
        )
    });
    c.bench_function("unify_twice", |b| {
        b.iter_batched_ref(
            || runner_from_query("1=1 and 2=2"),
            |runner| runner.run(),
            criterion::BatchSize::SmallInput,
        )
    });
}

pub fn not(c: &mut Criterion) {
    c.bench_function("not", |b| {
        b.iter_batched_ref(
            || runner_from_query("not false"),
            |runner| runner.run(),
            criterion::BatchSize::SmallInput,
        )
    });
    c.bench_function("double_not", |b| {
        b.iter_batched_ref(
            || runner_from_query("not (not true)"),
            |runner| runner.run(),
            criterion::BatchSize::SmallInput,
        )
    });
    c.bench_function("De_Morgan_not", |b| {
        b.iter_batched_ref(
            || runner_from_query("not (true or false)"),
            |runner| runner.run(),
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
            b.iter_batched_ref(
                || {
                    let mut runner = runner_from_query(&format!("fib({}, result)", n));
                    runner.load_str(policy).unwrap();
                    runner.expected_result(maplit::hashmap!(
                        sym!("result") => term!(fib(*n))
                    ));
                    runner
                },
                |runner| {
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
            b.iter_batched_ref(
                || {
                    let mut runner = runner_from_query(&format!("prime({})", n));
                    runner.load_str(policy).unwrap();
                    if prime(n) {
                        runner.expected_result(maplit::hashmap!())
                    }
                    runner
                },
                |runner| {
                    runner.run();
                },
                criterion::BatchSize::SmallInput,
            )
        });
    }

    group.finish();
}

/// Bench: create `TARGET` rules of the form `f(i)`
/// and measure the time to compute `f(i / 2)`
/// This basically measures the performance of the rule indexing
pub fn indexed_rules(c: &mut Criterion) {
    fn make_runner(n: usize) -> Runner {
        let mut runner = runner_from_query(&format!("f({})", n / 2));
        let mut policy = "f(0);".to_owned();
        for i in 1..=n {
            write!(policy, "f({});", i).unwrap();
        }
        runner.load_str(&policy).unwrap();
        runner.expected_result(Bindings::new());
        runner
    }

    let n_array = [100, 500, 1000, 10_000];

    let mut group = c.benchmark_group("indexed");
    for n in &n_array {
        group.bench_function(BenchmarkId::from_parameter(format!("{}", n)), |b| {
            b.iter_batched_ref(
                || make_runner(*n),
                |runner| {
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
        let mut policy = "f(0);".to_owned();
        for i in 1..=TARGET {
            write!(policy, "f({}) if f({});", i, i - 1).unwrap();
        }
        runner.load_str(&policy).unwrap();
        runner.expected_result(Bindings::new());
        runner
    }

    c.bench_function("many_rules", |b| {
        b.iter_batched_ref(
            make_runner,
            |runner| runner.run(),
            criterion::BatchSize::SmallInput,
        )
    });
}

fn load_policy(c: &mut Criterion) {
    let policy = include_str!("roles_policy.polar");
    c.bench_function("load_policy", |b| {
        b.iter(|| {
            let polar = Polar::new();
            polar.load_str(policy).unwrap()
        })
    });
}

criterion_group!(
    benches,
    simple_queries,
    many_rules,
    fib,
    prime,
    indexed_rules,
    not,
    load_policy,
);
