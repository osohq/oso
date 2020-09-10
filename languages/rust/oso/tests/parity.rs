use oso::{Class, HostClass, Oso, ToPolar};

macro_rules! res {
    ($res:expr) => {
        $res.unwrap().next().unwrap().unwrap();
    };
    (@not $res:expr) => {
        assert!($res.unwrap().next().is_none());
    };
}

#[derive(Clone)]
struct A {
    x: String,
}

impl HostClass for A {}

impl A {
    pub fn new(x: String) -> Self {
        Self { x }
    }

    pub fn foo(&self) -> i32 {
        -1
    }
}

// pub trait A {}

// TODO
// struct D;

// impl A for D {

// }

// oso.register_class(A)

pub mod b {
    #[derive(Clone, Default)]
    pub struct C {
        pub y: String,
    }

    impl C {
        pub fn new(y: String) -> Self {
            Self { y }
        }

        pub fn foo(&self) -> i32 {
            -1
        }
    }

    impl oso::HostClass for C {}
}

pub fn custom_c_constructor(y: String) -> b::C {
    b::C::new(y)
}

#[test]
fn test() {
    let mut oso = Oso::new();
    tracing_subscriber::fmt::init();

    Class::with_constructor(A::new)
        .name("A")
        .add_attribute_getter("x", |a_self: &A| a_self.x.clone())
        .add_method("foo", A::foo)
        .register(&mut oso)
        .unwrap();

    Class::with_constructor(b::C::new)
        .name("C")
        .add_attribute_getter("y", |c: &b::C| c.y.clone())
        .add_method("foo", b::C::foo)
        .register(&mut oso)
        .unwrap();

    let polar_file = std::env::var("CARGO_MANIFEST_DIR").unwrap() + "/../../../test/test.polar";
    println!("Loading: {}", polar_file);
    oso.load_file(&polar_file).unwrap();

    assert!(oso.is_allowed("a", "b", "c"));

    // Test that a built in string method can be called.
    oso.load_str(r#"?= x = "hello world!" and x.ends_with("world!");"#)
        .unwrap();

    assert_eq!(
        oso.load_str("missingSemicolon()").unwrap_err().to_string(),
        "hit the end of the file unexpectedly. Did you forget a semi-colon at line 1, column 19"
    );

    // let d = D("")
    // let args: Vec<&ToPolar> = vec![]
    // assert!(oso.query_rule("specializers", ))
    // assert list(oso.query_rule("specializers", D("hello"), B.C("hello")))
    res!(oso.query_rule("floatLists", None));
    res!(oso.query_rule("intDicts", None));
    res!(oso.query_rule("comparisons", None));
    res!(oso.query_rule("testForall", None));
    res!(oso.query_rule("testRest", None));
    let a = A::new("hello".to_string());
    res!(oso.query_rule("testMatches", vec![&a as &dyn ToPolar]));

    let c = b::C::new("hello".to_string());
    res!(oso.query_rule("testMethodCalls", vec![&a as &dyn ToPolar, &c]));
    res!(oso.query_rule("testOr", None));
    // res!(oso.query_rule("testHttpAndPathMapper", None));

    // Test that cut doesn't return anything.
    res!(@not oso.query_rule("testCut", None));

    // Test that a constant can be called.
    // oso.register_constant("Math", math);
    // oso.load_str("?= Math.factorial(5) == 120;").unwrap();

    // Test built-in type specializers.
    res!(oso.query(r#"builtinSpecializers(true, "Boolean")"#));
    res!(@not oso.query(r#"builtinSpecializers(false, "Boolean")"#));
    res!(oso.query(r#"builtinSpecializers(2, "Integer")"#));
    res!(oso.query(r#"builtinSpecializers(1, "Integer")"#));
    res!(@not oso.query(r#"builtinSpecializers(0, "Integer")"#));
    res!(@not oso.query(r#"builtinSpecializers(-1, "Integer")"#));
    res!(oso.query(r#"builtinSpecializers(1.0, "Float")"#));
    res!(@not oso.query(r#"builtinSpecializers(0.0, "Float")"#));
    res!(@not oso.query(r#"builtinSpecializers(-1.0, "Float")"#));
    res!(oso.query(r#"builtinSpecializers(["foo", "bar", "baz"], "List")"#));
    res!(@not oso.query(r#"builtinSpecializers(["bar", "foo", "baz"], "List")"#));
    res!(oso.query(r#"builtinSpecializers({foo: "foo"}, "Dictionary")"#));
    res!(@not oso.query(r#"builtinSpecializers({foo: "bar"}, "Dictionary")"#));
    res!(oso.query(r#"builtinSpecializers("foo", "String")"#));
    res!(@not oso.query(r#"builtinSpecializers("bar", "String")"#));
}
