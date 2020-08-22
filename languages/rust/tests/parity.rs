use oso::host::Class;
use oso::polar::Polar;

struct A {
    x: String,
}

impl A {
    pub fn new(x: String) -> Self {
        Self { x }
    }

    pub fn foo(&self) -> i32 {
        -1
    }
}

// pub trait A {}

struct D;

// impl A for D {

// }

// oso.register_class(A)

pub mod b {
    #[derive(Default)]
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
}

pub fn custom_c_constructor(y: String) -> b::C {
    b::C::new(y)
}

// oso.register_class(B.C, name="C", from_polar=custom_c_constructor)

#[test]
fn test() {
    let mut polar = Polar::new();

    let mut a_class = Class::with_constructor(A::new);
    a_class.add_attribute_getter("x", |a_self: &A| a_self.x.clone());
    a_class.add_method("foo", A::foo);
    polar
        .register_class(a_class, Some("A".to_string()))
        .unwrap();

    let mut c_class = Class::with_constructor(b::C::new);
    c_class.add_attribute_getter("y", |c: &b::C| c.y.clone());
    c_class.add_method("foo", b::C::foo);
    polar
        .register_class(c_class, Some("C".to_string()))
        .unwrap();

    let polar_file = std::env::var("CARGO_MANIFEST_DIR").unwrap() + "/../../test/test.polar";
    println!("Loading: {}", polar_file);
    polar.load_file(&polar_file).unwrap();
}

// polar_file = os.path.dirname(os.path.realpath(__file__)) + "/test.polar"
// oso.load_file(polar_file)

// assert oso.is_allowed("a", "b", "c")

// # Test that a built in string method can be called.
// oso.load_str("""?= x = "hello world!" and x.endswith("world!");""")

// # Test that a custom error type is thrown.
// exception_thrown = False
// try:
//     oso.load_str("missingSemicolon()")
// except UnrecognizedEOF as e:
//     exception_thrown = True
//     assert (
//         str(e)
//         == "hit the end of the file unexpectedly. Did you forget a semi-colon at line 1, column 19"
//     )
// assert exception_thrown

// assert list(oso.query_rule("specializers", D("hello"), B.C("hello")))
// assert list(oso.query_rule("floatLists"))
// assert list(oso.query_rule("intDicts"))
// assert list(oso.query_rule("comparisons"))
// assert list(oso.query_rule("testForall"))
// assert list(oso.query_rule("testRest"))
// assert list(oso.query_rule("testMatches", A("hello")))
// assert list(oso.query_rule("testMethodCalls", A("hello"), B.C("hello")))
// assert list(oso.query_rule("testOr"))
// assert list(oso.query_rule("testHttpAndPathMapper"))

// # Test that cut doesn't return anything.
// assert not list(oso.query_rule("testCut"))

// # Test that a constant can be called.
// oso.register_constant("Math", math)
// oso.load_str("?= Math.factorial(5) == 120;")

// # Test built-in type specializers.
// assert list(oso.query('builtinSpecializers(true, "Boolean")'))
// assert not list(oso.query('builtinSpecializers(false, "Boolean")'))
// assert list(oso.query('builtinSpecializers(2, "Integer")'))
// assert list(oso.query('builtinSpecializers(1, "Integer")'))
// assert not list(oso.query('builtinSpecializers(0, "Integer")'))
// assert not list(oso.query('builtinSpecializers(-1, "Integer")'))
// assert list(oso.query('builtinSpecializers(1.0, "Float")'))
// assert not list(oso.query('builtinSpecializers(0.0, "Float")'))
// assert not list(oso.query('builtinSpecializers(-1.0, "Float")'))
// assert list(oso.query('builtinSpecializers(["foo", "bar", "baz"], "List")'))
// assert not list(oso.query('builtinSpecializers(["bar", "foo", "baz"], "List")'))
// assert list(oso.query('builtinSpecializers({foo: "foo"}, "Dictionary")'))
// assert not list(oso.query('builtinSpecializers({foo: "bar"}, "Dictionary")'))
// assert list(oso.query('builtinSpecializers("foo", "String")'))
// assert not list(oso.query('builtinSpecializers("bar", "String")'))
