//! Benchmarks of blog post things

use oso::{Class, Oso, PolarClass};

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

pub fn rust_get_attribute(c: &mut Criterion) {
    #[derive(PolarClass)]
    struct Foo {
        x: u32,
    }

    c.bench_function("rust_get_attribute", |b| {
        b.iter_batched_ref(
            || {
                let mut oso = Oso::new();
                oso.register_class(
                    Class::builder::<Foo>()
                        .name("Foo")
                        .add_attribute_getter("x", |f| f.x)
                        .build(),
                )
                .unwrap();
                oso.load_str("foo_x_is_one(foo: Foo) if foo.x = 1;")
                    .unwrap();
                oso
            },
            |oso| {
                let test_foo = Foo { x: 1 };
                let mut query = oso.query_rule("foo_x_is_one", (test_foo,)).unwrap();
                let _ = query.next().expect("no results").expect("resulted in err");
            },
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
            child in grandparent.children() and
            grandchild in child.children() and
            grandchild.name = name;
    ";

    #[derive(Clone, PolarClass)]
    struct Person {
        #[polar(attribute)]
        name: &'static str,
    }

    impl Person {
        fn new() -> Self {
            Self { name: "alice" }
        }
    }

    let n_array = [100, 500, 1000];

    let mut group = c.benchmark_group("n_plus_one");
    for &n in &n_array {
        group.bench_function(BenchmarkId::from_parameter(n), |b| {
            b.iter_batched_ref(
                || {
                    let mut oso = Oso::new();
                    let person_class = Person::get_polar_class_builder()
                        .set_constructor(Person::new)
                        .add_iterator_method("children", move |person: &Person| match person.name {
                            // alice has N-1 children called bert and one called bpb
                            "alice" => std::iter::repeat(Person { name: "bert" })
                                .take(n - 1)
                                .chain(Some(Person { name: "bob" }))
                                .collect(),
                            // berts all have 1 child called charlie
                            "bert" => vec![Person { name: "charlie" }],
                            // bob all have 1 child called charlie
                            "bob" => vec![Person { name: "cora" }],
                            _ => vec![],
                        })
                        .build();
                    oso.register_class(person_class).unwrap();
                    oso.load_str(policy).unwrap();
                    oso.query("has_grandchild_called(new Person(), \"cora\")")
                        .unwrap()
                },
                |query| assert!(query.next_result().unwrap().is_ok()),
                criterion::BatchSize::SmallInput,
            )
        });
    }
    group.finish();
}

criterion_group!(benches, rust_get_attribute, n_plus_one_queries);
criterion_main!(benches);
