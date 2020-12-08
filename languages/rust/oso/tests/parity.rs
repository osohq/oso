use oso::{ClassBuilder, Oso, PolarClass};

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

impl PolarClass for A {}

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

    impl oso::PolarClass for C {}
}

#[test]
fn test() {
    let mut oso = Oso::new();
    tracing_subscriber::fmt::init();

    oso.register_class(
        ClassBuilder::with_constructor(A::new)
            .name("A")
            .add_attribute_getter("x", |a_self: &A| a_self.x.clone())
            .add_method("foo", A::foo)
            .build(),
    )
    .unwrap();

    oso.register_class(
        ClassBuilder::with_constructor(b::C::new)
            .name("C")
            .add_attribute_getter("y", |c: &b::C| c.y.clone())
            .add_method("foo", b::C::foo)
            .build(),
    )
    .unwrap();

    let polar_file = std::env::var("CARGO_MANIFEST_DIR").unwrap() + "/../../../test/test.polar";
    println!("Loading: {}", polar_file);
    oso.load_file(&polar_file).unwrap();

    assert!(oso.is_allowed("a", "b", "c").unwrap());

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
    res!(oso.query_rule("floatLists", ()));
    res!(oso.query_rule("intDicts", ()));
    res!(oso.query_rule("comparisons", ()));
    res!(oso.query_rule("testForall", ()));
    res!(oso.query_rule("testRest", ()));
    let a = A::new("hello".to_string());
    res!(oso.query_rule("testMatches", (a.clone(),)));

    let c = b::C::new("hello".to_string());
    res!(oso.query_rule("testMethodCalls", (a, c)));
    res!(oso.query_rule("testOr", ()));

    // Test that cut doesn't return anything.
    res!(@not oso.query_rule("testCut", ()));

    // Test that a constant can be called.
    // oso.register_constant(math, "MyMath");
    // oso.load_str("?= MyMath.factorial(5) == 120;").unwrap();

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

    // Rust ints do not have the denominator field
    // res!(oso.query(r#"builtinSpecializers(1, "IntegerWithFields")"#));
    res!(@not oso.query(r#"builtinSpecializers(2, "IntegerWithGarbageFields")"#));
    res!(@not oso.query(r#"builtinSpecializers({}, "DictionaryWithFields")"#));
    res!(@not oso.query(r#"builtinSpecializers({z: 1}, "DictionaryWithFields")"#));
    res!(oso.query(r#"builtinSpecializers({y: 1}, "DictionaryWithFields")"#));
}
