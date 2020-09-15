use maplit::hashmap;
use oso::{Class, HostClass, Oso, PolarClass, ToPolar};
use oso_derive::*;

struct OsoTest {
    oso: Oso,
}

impl OsoTest {
    fn new() -> Self {
        Self { oso: Oso::new() }
    }

    fn load_str(&mut self, policy: &str) {
        self.oso.load_str(policy).unwrap();
    }

    fn load_file(&mut self, here: &str, name: &str) {
        // hack because `file!()` starts from workspace root
        // https://github.com/rust-lang/cargo/issues/3946
        let folder = std::path::PathBuf::from(&here.replace("languages/rust/oso/", ""));
        let mut file = folder.parent().unwrap().to_path_buf();
        file.push(name);
        println!("{:?}", file);
        self.oso.load_file(file.to_str().unwrap()).unwrap();
    }

    fn query(&mut self, q: &str) -> Vec<oso::ResultSet> {
        let results = self.oso.query(q).unwrap();
        let mut result_vec = vec![];
        for r in results {
            result_vec.push(r.expect("result is an error"))
        }
        result_vec
    }

    fn query_err(&mut self, q: &str) -> String {
        let mut results = self.oso.query(q).unwrap();
        let err = results
            .next()
            .unwrap()
            .expect_err("query should return an error");
        err.to_string()
    }

    fn qvar<T: oso::FromPolar>(&mut self, q: &str, var: &str) -> Vec<T> {
        let res = self.query(q);
        res.into_iter()
            .map(|set| {
                set.get_typed(var)
                    .unwrap_or_else(|_| panic!("query: '{}', binding for '{}'", q, var))
            })
            .collect()
    }

    fn qeval(&mut self, q: &str) {
        let mut results = self.oso.query(q).unwrap();
        results
            .next()
            .expect("Query should have at least one result.")
            .unwrap();
    }

    fn qnull(&mut self, q: &str) {
        let mut results = self.oso.query(q).unwrap();
        assert!(results.next().is_none(), "Query shouldn't have any results");
    }

    fn qvar_one<T>(&mut self, q: &str, var: &str, expected: T)
    where
        T: oso::FromPolar + PartialEq<T> + std::fmt::Debug,
    {
        let mut res = self.qvar::<T>(q, var);
        assert_eq!(res.len(), 1, "expected exactly one result");
        assert_eq!(res.pop().unwrap(), expected);
    }
}

#[test]
fn test_anything_works() {
    let _ = tracing_subscriber::fmt::try_init();

    let mut test = OsoTest::new();
    test.load_str("f(1);");
    let results = test.query("f(x)");
    assert_eq!(results[0].get_typed::<u32>("x").unwrap(), 1);
    let results = test.query("f(y)");
    assert_eq!(results[0].get_typed::<u32>("y").unwrap(), 1);
}

#[test]
fn test_helpers() {
    let _ = tracing_subscriber::fmt::try_init();

    let mut test = OsoTest::new();
    test.load_file(file!(), "test_file.polar");
    assert_eq!(
        test.query("f(x)"),
        vec![
            hashmap! { "x" => 1, },
            hashmap! { "x" => 2, },
            hashmap! { "x" => 3, },
        ]
    );
    assert_eq!(test.qvar::<u32>("f(x)", "x"), [1, 2, 3]);
}

#[test]
fn test_data_conversions() {
    let _ = tracing_subscriber::fmt::try_init();

    let mut test = OsoTest::new();
    test.load_str(
        r#"
        a(1);
        b("two");
        c(true);
        d([1, "two", true]);"#,
    );
    test.qvar_one("a(x)", "x", 1);
    test.qvar_one("b(x)", "x", "two".to_string());
    test.qvar_one("c(x)", "x", true);
    use polar_core::terms::Value;
    // TODO: do we want to handle hlists better?
    // e.g. https://docs.rs/hlist/0.1.2/hlist/
    test.qvar_one(
        "d(x)",
        "x",
        vec![
            Value::Number(polar_core::terms::Numeric::Integer(1)),
            Value::String("two".to_string()),
            Value::Boolean(true),
        ],
    );
}

// This logic is changing. Updated when fixed
#[ignore]
#[test]
fn test_load_function() {
    let _ = tracing_subscriber::fmt::try_init();

    let mut test = OsoTest::new();
    test.load_file(file!(), "test_file.polar");
    test.load_file(file!(), "test_file.polar");
    assert_eq!(
        test.query("f(x)"),
        vec![
            hashmap! { "x" => 1, },
            hashmap! { "x" => 2, },
            hashmap! { "x" => 3, },
        ]
    );
    assert_eq!(test.qvar::<u32>("f(x)", "x"), [1, 2, 3]);

    test.oso.clear();
    test.load_file(file!(), "test_file.polar");
    test.load_file(file!(), "test_file_gx.polar");
    assert_eq!(
        test.query("f(x)"),
        vec![
            hashmap! { "x" => 1, },
            hashmap! { "x" => 2, },
            hashmap! { "x" => 3, },
        ]
    );
    assert_eq!(
        test.query("g(x)"),
        vec![
            hashmap! { "x" => 1, },
            hashmap! { "x" => 2, },
            hashmap! { "x" => 3, },
        ]
    );
}

#[test]
fn test_external() {
    let _ = tracing_subscriber::fmt::try_init();

    struct Foo {
        a: &'static str,
    }

    impl Foo {
        fn new(a: Option<&'static str>) -> Self {
            Self {
                a: a.unwrap_or("a"),
            }
        }

        #[allow(dead_code)]
        fn b(&self) -> impl Iterator<Item = &'static str> + Clone {
            vec!["b"].into_iter()
        }

        fn c() -> &'static str {
            "c"
        }

        fn d<X>(&self, x: X) -> X {
            x
        }

        fn e(&self) -> Vec<u32> {
            vec![1, 2, 3]
        }

        #[allow(dead_code)]
        fn f(&self) -> impl Iterator<Item = Vec<u32>> + Clone {
            vec![vec![1, 2, 3], vec![4, 5, 6], vec![7]].into_iter()
        }

        fn g(&self) -> std::collections::HashMap<String, &'static str> {
            hashmap!("hello".to_string() => "world")
        }

        fn h(&self) -> bool {
            true
        }
    }

    fn capital_foo() -> Foo {
        Foo::new(Some("A"))
    }

    let mut test = OsoTest::new();

    let foo_class = oso::Class::with_constructor(capital_foo)
        .name("Foo")
        .add_attribute_getter("a", |receiver: &Foo| receiver.a)
        // .add_method("b", |receiver: &Foo| oso::host::PolarIter(receiver.b()))
        .add_class_method("c", Foo::c)
        .add_method::<_, _, u32>("d", Foo::d)
        .add_method("e", Foo::e)
        // .add_method("f", |receiver: &Foo| oso::host::PolarIter(receiver.f()))
        .add_method("g", Foo::g)
        .add_method("h", Foo::h)
        .build();
    test.oso.register_class(foo_class).unwrap();

    test.qvar_one("new Foo().a = x", "x", "A".to_string());
    test.query_err("new Foo().a() = x");

    // test.query_err("new Foo().b = x");
    // test.qvar_one("new Foo().b() = x", "x", vec!["b".to_string()]);

    test.qvar_one("Foo.c() = x", "x", "c".to_string());
    test.qvar_one("new Foo().d(1) = x", "x", 1);
    test.query_err("new Foo().d(\"1\") = x");
    test.qvar_one("new Foo() = f and f.a = x", "x", "A".to_string());
    test.qvar_one("new Foo().e() = x", "x", vec![1, 2, 3]);
    // test.qvar_one(
    //     "new Foo().f() = x",
    //     "x",
    //     vec![vec![1, 2, 3], vec![4, 5, 6], vec![7]],
    // );
    test.qvar_one("new Foo().g().hello = x", "x", "world".to_string());
    test.qvar_one("new Foo().h() = x", "x", true);
}

#[test]
//#[allow(clippy::redundant-closure)]
fn test_methods() {
    use std::default::Default;

    let _ = tracing_subscriber::fmt::try_init();

    #[derive(PolarClass, Clone)]
    struct Foo {
        #[polar(attribute)]
        a: String,
    }

    #[derive(PolarClass, Debug, Clone)]
    struct Bar {
        #[polar(attribute)]
        b: String,
    }

    impl Default for Bar {
        fn default() -> Self {
            Self {
                b: "default".to_owned(),
            }
        }
    }

    impl Bar {
        pub fn bar(&self) -> Bar {
            self.clone()
        }

        pub fn foo(&self) -> Foo {
            Foo { a: self.b.clone() }
        }
    }
    let mut test = OsoTest::new();
    test.oso.register_class(Foo::get_polar_class()).unwrap();
    #[allow(clippy::redundant_closure)]
    // @TODO: Not sure how to get the default call to typecheck without the closure wrapper.
    test.oso
        .register_class(
            Bar::get_polar_class_builder()
                .set_constructor(|| Bar::default())
                .add_method("foo", |bar: &Bar| bar.foo())
                .add_method("bar", |bar: &Bar| bar.bar())
                .add_method("clone", Clone::clone)
                .build(),
        )
        .unwrap();

    // Test chaining
    test.qvar_one(r#"new Bar().bar().foo().a = x"#, "x", "default".to_string());
    // Test trait method.
    test.qvar_one(r#"new Bar().clone().b = x"#, "x", "default".to_string());
}

#[test]
fn test_macros() {
    let _ = tracing_subscriber::fmt::try_init();

    #[derive(PolarClass)]
    #[polar(class_name = "Bar")]
    struct Foo {
        #[polar(attribute)]
        a: String,
        #[polar(attribute)]
        b: String,
    }

    impl Foo {
        fn new(a: String) -> Self {
            Self {
                a,
                b: "b".to_owned(),
            }
        }

        fn goodbye() -> Self {
            Self {
                a: "goodbye".to_owned(),
                b: "b".to_owned(),
            }
        }
    }

    let mut test = OsoTest::new();
    test.oso
        .register_class(
            Foo::get_polar_class_builder()
                .set_constructor(Foo::new)
                .build(),
        )
        .unwrap();

    test.query(r#"new Bar("hello") = x"#);
    test.qvar_one(r#"new Bar("hello").a = x"#, "x", "hello".to_string());
    test.qvar_one(r#"new Bar("hello").b = x"#, "x", "b".to_string());

    let class_builder = Foo::get_polar_class_builder();
    let class = class_builder
        .name("Baz")
        .set_constructor(Foo::goodbye)
        .add_method("world", |receiver: &Foo| format!("{} world", receiver.a))
        .build();
    test.oso.register_class(class).unwrap();

    test.qvar_one(r#"new Baz().world() = x"#, "x", "goodbye world".to_string());
}

#[test]
fn test_tuple_structs() {
    let _ = tracing_subscriber::fmt::try_init();
    #[derive(PolarClass)]
    struct Foo(i32, i32);

    impl Foo {
        fn new(a: i32, b: i32) -> Self {
            Self(a, b)
        }
    }

    // @TODO: In the future when we can reason about which attributes are accessible types
    // we can auto generate these accessors too. For now we have to rely on the attribute for
    // fields and manually doing it for tuple structs.
    // Also foo.0 isn't valid polar syntax so if we wanted something like that to work in general for "tuple like objects
    // that requires a bigger change.
    let mut test = OsoTest::new();
    test.oso
        .register_class(
            Foo::get_polar_class_builder()
                .set_constructor(Foo::new)
                .add_attribute_getter("i0", |rcv: &Foo| rcv.0)
                .add_attribute_getter("i1", |rcv: &Foo| rcv.1)
                .build(),
        )
        .unwrap();

    test.qvar_one(r#"foo = new Foo(1,2) and foo.i0 + foo.i1 = x"#, "x", 3);
}

#[test]
fn test_results_and_options() {
    let _ = tracing_subscriber::fmt::try_init();

    #[derive(PolarClass)]
    struct Foo;

    impl Foo {
        fn new() -> Self {
            Self
        }

        fn ok(&self) -> Result<i32, String> {
            Ok(1)
        }

        fn err(&self) -> Result<i32, &'static str> {
            Err("Some sort of error")
        }

        fn some(&self) -> Option<i32> {
            Some(1)
        }

        fn none(&self) -> Option<i32> {
            None
        }
    }

    let mut test = OsoTest::new();
    test.oso
        .register_class(
            Foo::get_polar_class_builder()
                .set_constructor(Foo::new)
                .add_method("ok", Foo::ok)
                .add_method("err", Foo::err)
                .add_method("some", Foo::some)
                .add_method("none", Foo::none)
                .build(),
        )
        .unwrap();

    test.qvar_one(r#"new Foo().ok() = x"#, "x", 1);
    test.query_err("new Foo().err()");
    test.qvar_one(r#"new Foo().some() = x"#, "x", 1);
    let results = test.query("new Foo().none()");
    assert!(results.is_empty());
}

// TODO: dhatch see if there is a relevant test to port.
#[test]
fn test_unify_externals() {
    let mut test = OsoTest::new();

    #[derive(PartialEq, Clone, Debug)]
    struct Foo {
        x: i64,
    }

    impl HostClass for Foo {};
    impl Foo {

        fn new(x: i64) -> Self {
            Self { x }
        }
    }

    let foo_class = Class::with_constructor(Foo::new)
        .name("Foo")
        .add_attribute_getter("x", |this: &Foo| this.x)
        .with_equality_check()
        .build();

    test.oso.register_class(foo_class).unwrap();

    test.load_str("foos_equal(a, b) if a = b;");

    // Test with instantiated in polar.
    test.qeval("foos_equal(new Foo(1), new Foo(1))");
    test.qnull("foos_equal(new Foo(1), new Foo(2))");

    let a = Foo::new(1);
    let b = Foo::new(1);
    assert_eq!(a, b);

    // TODO this interface is not convenient or easy to use due to all the casting. Maybe needs a macro?
    let mut results = test
        .oso
        .query_rule("foos_equal", vec![&a as &dyn ToPolar, &b as &dyn ToPolar])
        .unwrap();
    results.next().expect("At least one result").unwrap();
}
