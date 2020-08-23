use maplit::hashmap;
use oso::polar::Polar;

struct PolarTest {
    polar: Polar,
}

impl PolarTest {
    fn new() -> Self {
        Self {
            polar: Polar::new(),
        }
    }

    fn load_str(&mut self, policy: &str) {
        self.polar.load_str(policy).unwrap();
    }

    fn load_file(&mut self, here: &str, name: &str) {
        // hack because `file!()` starts from workspace root
        // https://github.com/rust-lang/cargo/issues/3946
        let folder = std::path::PathBuf::from(&here.replace("languages/rust/", ""));
        let mut file = folder.parent().unwrap().to_path_buf();
        file.push(name);
        println!("{:?}", file);
        self.polar.load_file(file.to_str().unwrap()).unwrap();
    }

    fn query(&mut self, q: &str) -> Vec<oso::query::ResultSet> {
        let mut results = self.polar.query(q).unwrap();
        let mut result_vec = vec![];
        while let Some(r) = results.next() {
            result_vec.push(r.expect("result is an error"))
        }
        result_vec
    }

    fn query_err(&mut self, q: &str) -> String {
        let mut results = self.polar.query(q).unwrap();
        let err = results
            .next()
            .unwrap()
            .expect_err("query should return an error");
        err.to_string()
    }

    fn qvar<T: oso::host::FromPolar>(&mut self, q: &str, var: &str) -> Vec<T> {
        let res = self.query(q);
        res.into_iter()
            .map(|set| {
                set.get(var)
                    .expect(&format!("query: '{}', binding for '{}'", q, var))
            })
            .collect()
    }

    fn qvar_one<T>(&mut self, q: &str, var: &str, expected: T)
    where
        T: oso::host::FromPolar + PartialEq<T> + std::fmt::Debug,
    {
        let mut res = self.qvar::<T>(q, var);
        assert_eq!(res.len(), 1, "expected exactly one result");
        assert_eq!(res.pop().unwrap(), expected);
    }
}

#[test]
fn test_anything_works() {
    let mut test = PolarTest::new();
    test.load_str("f(1);");
    let results = test.query("f(x)");
    assert_eq!(results[0].get::<u32>("x"), Some(1));
    let results = test.query("f(y)");
    assert_eq!(results[0].get::<u32>("y"), Some(1));
}

#[test]
fn test_helpers() {
    let mut test = PolarTest::new();
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
    let mut test = PolarTest::new();
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
    use polar_core::types::Value;
    // TODO: do we want to handle hlists better?
    // e.g. https://docs.rs/hlist/0.1.2/hlist/
    test.qvar_one(
        "d(x)",
        "x",
        vec![
            Value::Number(polar_core::types::Numeric::Integer(1)),
            Value::String("two".to_string()),
            Value::Boolean(true),
        ],
    );
}

// This logic is changing. Updated when fixed
#[ignore]
#[test]
fn test_load_function() {
    let mut test = PolarTest::new();
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

    test.polar.clear();
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
    struct Foo {
        a: &'static str,
    }

    impl Foo {
        fn new(a: Option<&'static str>) -> Self {
            Self {
                a: a.unwrap_or("a"),
            }
        }

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

    let mut test = PolarTest::new();

    oso::host::Class::with_constructor(capital_foo)
        .name("Foo")
        .add_attribute_getter("a", |receiver: &Foo| receiver.a)
        .add_method("b", |receiver: &Foo| oso::host::PolarIter(receiver.b()))
        .add_class_method("c", Foo::c)
        .add_method::<_, _, u32>("d", Foo::d)
        .add_method("e", Foo::e)
        .add_method("f", |receiver: &Foo| oso::host::PolarIter(receiver.f()))
        .add_method("g", Foo::g)
        .add_method("h", Foo::h)
        .register(&mut test.polar)
        .unwrap();

    test.qvar_one("new Foo().a = x", "x", "A".to_string());
    test.query_err("new Foo().a() = x");

    test.query_err("new Foo().b = x");
    test.qvar_one("new Foo().b() = x", "x", vec!["b".to_string()]);

    // TODO: Register Foo as a constant
    // test.qvar_one("Foo.c() = x", "x", "c".to_string());
    test.qvar_one("new Foo() = f and f.a = x", "x", "A".to_string());
    test.qvar_one("new Foo().e() = x", "x", vec![1, 2, 3]);
    test.qvar_one(
        "new Foo().f() = x",
        "x",
        vec![vec![1, 2, 3], vec![4, 5, 6], vec![7]],
    );
    test.qvar_one("new Foo().g().hello = x", "x", "world".to_string());
    test.qvar_one("new Foo().h() = x", "x", true);
}
