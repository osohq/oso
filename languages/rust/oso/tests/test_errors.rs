/// Tests that errors are raised & correct.
mod common;

use common::OsoTest;
use oso::{OsoError, PolarClass, Value};
use polar_core::terms::Symbol;

// TODO in all tests, check type of error & message

/// Test that external unification on raises error on:
/// - Same type that doesn't support unification
/// - Type that does unified with a different type
#[test]
fn test_unify_external_not_supported() -> oso::Result<()> {
    common::setup();

    #[derive(PolarClass)]
    struct Foo(pub i64);

    let mut oso = OsoTest::new();

    oso.oso.register_class(Foo::get_polar_class())?;

    oso.load_str("unify(x, y) if x = y;");

    // Type that doesn't support unification.
    let mut query = oso.oso.query_rule("unify", (Foo(1), Foo(1)))?;
    let error = query.next().unwrap().unwrap_err();
    assert!(
        matches!(
            &error,
            OsoError::UnsupportedOperation {
                operation,
                type_name
            } if operation == "equals" && type_name == "Foo"),
        "{} doesn't match expected error",
        error
    );

    // Type that does support unification with a type that doesn't.
    #[derive(PolarClass, PartialEq)]
    struct EqFoo(i64);

    impl PartialEq<Foo> for EqFoo {
        fn eq(&self, other: &Foo) -> bool {
            self.0 == other.0
        }
    }

    let eq_foo_class = EqFoo::get_polar_class_builder()
        .with_equality_check()
        .build();

    oso.oso.register_class(eq_foo_class)?;

    let mut query = oso.oso.query_rule("unify", (EqFoo(1), Foo(1)))?;
    let error = query.next().unwrap().unwrap_err();

    // TODO definitely need stack traces, these would be hard to diagnose
    // otherwise.
    assert!(
        matches!(
            &error,
            OsoError::TypeError(oso::errors::TypeError {
                expected
            }) if expected == "test_errors::test_unify_external_not_supported::EqFoo"),
        "{} doesn't match expected error",
        error
    );

    let mut query = oso.oso.query_rule("unify", (EqFoo(1), 1))?;
    let error = query.next().unwrap().unwrap_err();
    assert!(
        matches!(
            &error,
            OsoError::TypeError(oso::errors::TypeError {
                expected
            }) if expected == "test_errors::test_unify_external_not_supported::EqFoo"),
        "{} doesn't match expected error",
        error
    );

    Ok(())
}

/// Test that lookup of attribute that doesn't exist raises error.
#[test]
fn test_attribute_does_not_exist() -> oso::Result<()> {
    common::setup();

    let mut oso = OsoTest::new();

    #[derive(PolarClass)]
    struct Foo;

    oso.oso.register_class(Foo::get_polar_class())?;
    oso.load_str("getattr(x, y, val) if val = x.(y);");

    // TODO dhatch: Query API for variables needs improvement.
    let mut query = oso.oso.query_rule(
        "getattr",
        (Foo, "bar", Value::Variable(Symbol("a".to_owned()))),
    )?;
    query
        .next()
        .unwrap()
        .expect_err("Attribute does not exist.");

    Ok(())
}

/// Test that lookup of method that doesn't exist raises error.
#[test]
fn test_method_does_not_exist() -> oso::Result<()> {
    common::setup();

    let mut oso = OsoTest::new();

    #[derive(PolarClass)]
    struct Foo;

    impl Foo {
        fn a(&self) -> i64 {
            1
        }
    }

    let foo_class = Foo::get_polar_class_builder()
        .add_method("a", Foo::a)
        .build();

    oso.oso.register_class(foo_class)?;
    oso.load_str("getmethod_b(x, val) if val = x.b();");

    let mut query = oso.oso.query_rule("getmethod_b", (Foo, 1))?;
    query.next().unwrap().expect_err("Should return error");

    Ok(())
}

/// Test that lookup of class method that doesn't exist raises error.
#[test]
fn test_class_method_does_not_exist() -> oso::Result<()> {
    common::setup();

    let mut oso = OsoTest::new();

    #[derive(PolarClass)]
    struct Foo;

    impl Foo {
        fn a() -> i64 {
            1
        }
    }

    let foo_class = Foo::get_polar_class_builder()
        .add_class_method("a", Foo::a)
        .build();

    oso.oso.register_class(foo_class)?;
    oso.load_str("getmethod_b(val) if val = Foo.b();");

    let mut query = oso.oso.query_rule("getmethod_b", (1,))?;
    query.next().unwrap().expect_err("Should return error");

    Ok(())
}

/// Test that method call with incorrect type raises error:
/// - Wrong type of arguments
/// - Arguments that are not registered
#[test]
fn test_wrong_argument_types() {
    common::setup();

    let mut oso = OsoTest::new();

    #[derive(PolarClass)]
    struct Foo;

    impl Foo {
        fn a(&self) -> i64 {
            1
        }

        fn bar(&self, _bar: Bar) -> i64 {
            2
        }

        fn bar_x(&self, _x: i64, _bar: Bar) -> i64 {
            3
        }

        fn int(&self, _x: u8) -> i64 {
            4
        }
    }

    // TODO (dhatch): Note for memory mgmt PR. Clone is required to use a type
    // as a method argument! But not otherwise (see Foo doesn't need Clone).
    #[derive(PolarClass, Clone)]
    struct Bar;

    #[derive(PolarClass)]
    struct Unregistered;

    let foo_class = Foo::get_polar_class_builder()
        .add_method("a", Foo::a)
        .add_method("bar", Foo::bar)
        .add_method("bar_x", Foo::bar_x)
        .add_method("int", Foo::int)
        .build();

    oso.oso.register_class(foo_class).unwrap();
    oso.oso.register_class(Bar::get_polar_class()).unwrap();

    oso.load_str("a(f, v) if v = f.a();");
    oso.load_str("bar(f, arg) if _v = f.bar(arg);");
    oso.load_str("bar_x(f, arg, arg1) if _v = f.bar_x(arg, arg1);");
    oso.load_str("int(f, arg) if _v = f.int(arg, arg1);");

    let mut query = oso.oso.query_rule("a", (Foo, 1)).unwrap();
    assert_eq!(query.next().unwrap().unwrap().keys().count(), 0);

    let mut query = oso.oso.query_rule("bar", (Foo, Bar)).unwrap();
    assert_eq!(query.next().unwrap().unwrap().keys().count(), 0);

    // Wrong type of argument.
    let mut query = oso.oso.query_rule("bar", (Foo, 1)).unwrap();
    assert!(query.next().unwrap().is_err());

    // Wrong type of argument.
    let mut query = oso.oso.query_rule("bar", (Foo, Foo)).unwrap();
    assert!(query.next().unwrap().is_err());

    // Unregistered argument.
    let mut query = oso.oso.query_rule("bar", (Foo, Unregistered)).unwrap();
    assert!(query.next().unwrap().is_err());

    let mut query = oso.oso.query_rule("bar_x", (Foo, 1, Bar)).unwrap();
    assert_eq!(query.next().unwrap().unwrap().keys().count(), 0);

    // Wrong type of argument.
    let mut query = oso.oso.query_rule("bar_x", (Foo, Foo, 1)).unwrap();
    assert!(query.next().unwrap().is_err());

    // Wrong type of argument.
    let mut query = oso.oso.query_rule("bar_x", (Foo, Bar, 1)).unwrap();
    assert!(query.next().unwrap().is_err());

    // Wrong type of argument.
    let mut query = oso.oso.query_rule("bar_x", (Foo, Bar, Bar)).unwrap();
    assert!(query.next().unwrap().is_err());

    // Unregistered argument.
    let mut query = oso
        .oso
        .query_rule("bar_x", (Foo, Bar, Unregistered))
        .unwrap();
    assert!(query.next().unwrap().is_err());

    // Wrong type of argument.
    let mut query = oso.oso.query_rule("int", (Foo, 0)).unwrap();
    assert_eq!(query.next().unwrap().unwrap().keys().count(), 0);

    // Wrong type of argument.
    let mut query = oso.oso.query_rule("int", (Foo, Bar)).unwrap();
    assert!(query.next().unwrap().is_err());

    // Wrong type of argument.
    let mut query = oso.oso.query_rule("int", (Foo, 0.)).unwrap();
    assert!(query.next().unwrap().is_err());

    // Out of bound argument.
    let mut query = oso.oso.query_rule("int", (Foo, i64::MAX)).unwrap();
    assert!(query.next().unwrap().is_err());

    // Out of bound argument.
    let mut query = oso.oso.query_rule("int", (Foo, -1 as i8)).unwrap();
    assert!(query.next().unwrap().is_err());

    // Out of bound argument.
    let mut query = oso.oso.query("int(-1)").unwrap();
    assert!(query.next().unwrap().is_err());

    // Out of bound argument.
    let mut query = oso.oso.query("int(256)").unwrap();
    assert!(query.next().unwrap().is_err());
}

/// Test that constructor call with incorrect type raises error:
/// - Wrong type of arguments
/// - Arguments that are not registered
#[test]
fn test_wrong_argument_types_constructor() {
    common::setup();

    let mut oso = OsoTest::new();

    #[derive(PolarClass)]
    struct Foo;

    // TODO (dhatch): This must be clone do to constraints on ToPolar.
    #[derive(PolarClass, Clone)]
    struct Bar;

    impl Foo {
        fn new(_bar: Bar) -> Self {
            Foo
        }
    }

    let foo_class = Foo::get_polar_class_builder()
        .set_constructor(Foo::new)
        .build();

    oso.oso.register_class(foo_class).unwrap();
    oso.oso.register_class(Bar::get_polar_class()).unwrap();

    oso.load_str("new_foo(val) if _v = new Foo(val);");

    let mut query = oso.oso.query_rule("new_foo", (Bar,)).unwrap();
    assert_eq!(query.next().unwrap().unwrap().keys().count(), 0);

    let mut query = oso.oso.query_rule("new_foo", (1,)).unwrap();
    assert!(query.next().unwrap().is_err());

    let mut query = oso.oso.query_rule("new_foo", (Foo,)).unwrap();
    assert!(query.next().unwrap().is_err());
}

/// Test match with non-existent attributes does not raise error
#[test]
fn test_match_attribute_does_not_exist() {
    common::setup();

    let mut oso = OsoTest::new();

    #[derive(PolarClass)]
    struct Foo {
        #[polar(attribute)]
        x: i64,
    };

    impl Foo {
        fn new() -> Self {
            Foo { x: 1 }
        }
    }

    let foo_class = Foo::get_polar_class_builder()
        .set_constructor(Foo::new)
        .build();

    oso.oso.register_class(foo_class).unwrap();

    oso.load_str("foo(d) if d matches Foo{x: 1};");
    oso.load_str("no_match_foo(d) if not d matches Foo{not_an_attr: 1};");
    oso.qeval("foo(new Foo())");
    oso.qeval("no_match_foo(new Foo())");
}

/// Test that match with class that doesn't exist raises error
#[test]
fn test_match_non_existent_class() {
    common::setup();

    let mut oso = OsoTest::new();

    #[derive(PolarClass)]
    struct Foo {
        #[polar(attribute)]
        x: i64,
    };

    impl Foo {
        fn new() -> Self {
            Foo { x: 1 }
        }
    }

    let foo_class = Foo::get_polar_class_builder()
        .set_constructor(Foo::new)
        .build();

    oso.oso.register_class(foo_class).unwrap();

    oso.load_str("foo(d) if d matches Bar{x: 1};");
    oso.query_err("foo(new Foo())");
}

/// Test that incorrect number of arguments raises error:
/// - Incorrect number of arguments on method
/// - Incorrect number of arguments for constructor
#[test]
fn test_wrong_argument_arity() -> oso::Result<()> {
    common::setup();

    let mut oso = OsoTest::new();

    #[derive(PolarClass)]
    struct Foo;

    impl Foo {
        fn a(&self, x: i64) -> i64 {
            x
        }
    }

    let foo_class = Foo::get_polar_class_builder()
        .add_method("a", Foo::a)
        .build();

    oso.oso.register_class(foo_class)?;

    oso.load_str("getmethod_a1(x, val) if val = x.a(val);");
    oso.load_str("getmethod_a2(x, val, val2) if val = x.a(val, val2);");
    oso.load_str("getmethod_a0(x) if val = x.a();");

    // Correct number of arguments
    let mut query = oso.oso.query_rule("getmethod_a1", (Foo, 1))?;
    assert_eq!(query.next().unwrap().unwrap().keys().count(), 0);

    // Too many arguments
    let mut query = oso.oso.query_rule("getmethod_a2", (Foo, 1, 2))?;
    assert!(query.next().unwrap().is_err());

    // Too few arguments
    let mut query = oso.oso.query_rule("getmethod_a0", (Foo,))?;
    assert!(query.next().unwrap().is_err());

    Ok(())
}

/// Test that constructing a class that is not registered raises error
#[test]
fn test_class_does_not_exist() {
    common::setup();

    let mut oso = OsoTest::new();

    #[derive(PolarClass)]
    struct Foo {
        #[polar(attribute)]
        x: i64,
    };

    impl Foo {
        fn new() -> Self {
            Foo { x: 1 }
        }
    }

    let foo_class = Foo::get_polar_class_builder()
        .set_constructor(Foo::new)
        .build();

    oso.oso.register_class(foo_class).unwrap();

    oso.load_str("bar(b) if b = new Bar()");
    let mut query = oso.oso.query("bar(b)").unwrap();
    assert!(query.next().unwrap().is_err());
}

/// Test that using keyword arguments for constructor raises error:
/// - Keyword args only
/// - Mixed parameters
#[test]
fn test_constructor_keyword_arguments_error() {
    common::setup();

    let mut oso = OsoTest::new();

    #[derive(PolarClass)]
    struct Foo {
        #[polar(attribute)]
        x: i64,
    };

    impl Foo {
        fn new(x: i64) -> Self {
            Foo { x }
        }
    }

    let foo_class = Foo::get_polar_class_builder()
        .set_constructor(Foo::new)
        .build();

    oso.oso.register_class(foo_class).unwrap();

    let mut query = oso.oso.query("x = new Foo(1)").unwrap();
    assert_eq!(query.next().unwrap().unwrap().keys().count(), 1);

    let mut query = oso.oso.query("x = new Foo(x: 1)").unwrap();
    assert!(query.next().unwrap().is_err());

    let mut query = oso.oso.query("x = new Foo(1, x: 1)").unwrap();
    assert!(query.next().unwrap().is_err());
}

/// Test that using keyword arguments for method raises error:
/// - Keyword args only
/// - Mixed parameters
#[test]
fn test_method_keyword_arguments_error() -> oso::Result<()> {
    common::setup();

    let mut oso = OsoTest::new();

    #[derive(PolarClass)]
    struct Foo {
        #[polar(attribute)]
        x: i64,
    };

    impl Foo {
        fn new() -> Self {
            Foo { x: 1 }
        }

        fn a(&self, x: i64) -> i64 {
            x
        }
    }

    let foo_class = Foo::get_polar_class_builder()
        .set_constructor(Foo::new)
        .add_method("a", Foo::a)
        .build();

    oso.oso.register_class(foo_class)?;

    let mut query = oso.oso.query("x = new Foo(1).a(1)").unwrap();
    assert_eq!(query.next().unwrap()?.get_typed::<i64>("x")?, 1);

    let mut query = oso.oso.query("x = new Foo(1).a(x: 1)").unwrap();
    assert!(query.next().unwrap().is_err());

    let mut query = oso.oso.query("x = new Foo(1).a(1, x: 1)").unwrap();
    assert!(query.next().unwrap().is_err());

    Ok(())
}

/// Test operator raises not implemented error.
#[test]
fn test_operator_unimplemented() -> oso::Result<()> {
    common::setup();

    let mut oso = OsoTest::new();

    #[derive(PolarClass, PartialOrd, PartialEq)]
    struct Foo(i64);

    let foo_class = Foo::get_polar_class_builder().with_equality_check().build();

    oso.oso.register_class(foo_class)?;

    oso.load_str("lt(a, b) if a < b;");
    oso.qeval("lt(1, 2)");

    assert!(Foo(0) < Foo(1));
    let mut query = oso.oso.query_rule("lt", (Foo(0), Foo(1))).unwrap();
    assert!(query.next().unwrap().is_err());

    Ok(())
}

// TODO (dhatch): Test errors for application method failures.

// TODO (dhatch): What would happen for something like
// val matches Foo { x: 1 } where val.x is not an integer.
//
// This would raise a type error (if we did one-sided external unification,
// but we want the matches to just fail.  This wouldn't be caught by the
// current application error implementation.
