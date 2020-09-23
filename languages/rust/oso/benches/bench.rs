//! Benchmarks of blog post things

use oso::{Class, Oso, ToPolar};

use criterion::{criterion_group, criterion_main, Criterion};

pub fn rust_get_attribute(c: &mut Criterion) {
    struct Foo {
        x: u32,
    }
    impl ToPolar for Foo {}

    c.bench_function("rust_get_attribute", |b| {
        b.iter_batched_ref(
            || {
                let mut oso = Oso::new();
                oso.load_str("foo_x_is_one(foo: Foo) if foo.x = 1;")
                    .unwrap();
                oso.register_class(
                    Class::builder::<Foo>()
                        .name("Foo")
                        .add_attribute_getter("x", |f| f.x)
                        .build(),
                )
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

criterion_group!(benches, rust_get_attribute);
criterion_main!(benches);
