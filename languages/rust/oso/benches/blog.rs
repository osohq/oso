//! Benchmarks of blog post things

use oso::{Class, Oso, ToPolar};

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};

pub fn print_string(c: &mut Criterion) {
    c.bench_function("print_string", |b| {
        b.iter_batched(
            || "Hello, World!".to_string(),
            |mut s| {
                s.make_ascii_uppercase();
                s
            },
            BatchSize::SmallInput,
        )
    });
}

pub fn print_string_dyn(c: &mut Criterion) {
    c.bench_function("print_string_dyn", |b| {
        b.iter_batched(
            || {
                let s = "Hello, World!".to_string();
                let boxed: Box<dyn std::any::Any> = Box::new(s);
                boxed
            },
            |boxed| {
                let mut recovered: Box<String> = boxed.downcast().unwrap();
                recovered.make_ascii_uppercase();
                recovered
            },
            BatchSize::SmallInput,
        )
    });
}

pub fn get_attribute(c: &mut Criterion) {
    struct Foo {
        x: u32,
    }
    impl ToPolar for Foo {}

    c.bench_function("get_attribute", |b| {
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

criterion_group!(benches, get_attribute, print_string, print_string_dyn);
criterion_main!(benches);
