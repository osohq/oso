#![allow(clippy::too_many_arguments)]
/// Tests that are unique to the Rust implementation of oso, testing things like
/// rust class handling.
use maplit::hashmap;
use thiserror::Error;

use oso::{ClassBuilder, PolarClass};

mod common;

use common::OsoTest;

#[test]
fn test_anything_works() {
    common::setup();

    let mut test = OsoTest::new();
    test.load_str("f(1);");
    let results = test.query("f(x)");
    assert_eq!(results[0].get_typed::<u32>("x").unwrap(), 1);
    let results = test.query("f(y)");
    assert_eq!(results[0].get_typed::<u32>("y").unwrap(), 1);
}

#[test]
fn test_helpers() {
    common::setup();

    let mut test = OsoTest::new();
    test.load_file(file!(), "test_file.polar").unwrap();
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
    common::setup();

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

    use oso::PolarValue;

    // TODO: do we want to handle hlists better?
    // e.g. https://docs.rs/hlist/0.1.2/hlist/
    let mut results = test.query("d(x)");
    let first = results.pop().unwrap();
    let mut x = first.get_typed::<Vec<PolarValue>>("x").unwrap();
    assert_eq!(i64::try_from(x.remove(0)).unwrap(), 1);
    assert_eq!(String::try_from(x.remove(0)).unwrap(), "two");
    assert!(bool::try_from(x.remove(0)).unwrap());
}

// This logic is changing. Updated when fixed
#[ignore]
#[test]
fn test_load_function() {
    common::setup();

    let mut test = OsoTest::new();
    test.load_file(file!(), "test_file.polar").unwrap();
    test.load_file(file!(), "test_file.polar").unwrap();
    assert_eq!(
        test.query("f(x)"),
        vec![
            hashmap! { "x" => 1, },
            hashmap! { "x" => 2, },
            hashmap! { "x" => 3, },
        ]
    );
    assert_eq!(test.qvar::<u32>("f(x)", "x"), [1, 2, 3]);

    assert!(matches!(test.oso.clear_rules(), Ok(())));
    test.load_file(file!(), "test_file.polar").unwrap();
    test.load_file(file!(), "test_file_gx.polar").unwrap();
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
fn test_type_mismatch_fails_unification() {
    common::setup();

    #[derive(Eq, PartialEq, PolarClass, Clone, Default)]
    struct Foo {}
    #[derive(Eq, PartialEq, PolarClass, Clone, Default)]
    struct Bar {}

    let mut test = OsoTest::new();
    test.oso
        .register_class(
            ClassBuilder::<Foo>::with_default()
                .with_equality_check()
                .build(),
        )
        .unwrap();

    test.oso
        .register_class(
            ClassBuilder::<Bar>::with_default()
                .with_equality_check()
                .build(),
        )
        .unwrap();

    test.qnull("new Foo() = new Bar()");
    test.qnull("new Foo() = nil");
    let rs = test.query("not new Foo() = nil");
    assert_eq!(rs.len(), 1, "expected one result");
    assert!(rs[0].is_empty(), "expected empty result");
}

#[test]
fn test_external() {
    common::setup();

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

        fn g(&self) -> std::collections::HashMap<&'static str, &'static str> {
            hashmap!("hello" => "world")
        }

        fn h(&self) -> bool {
            true
        }
    }

    fn capital_foo() -> Foo {
        Foo::new(Some("A"))
    }

    let mut test = OsoTest::new();

    let foo_class = oso::ClassBuilder::with_constructor(capital_foo)
        .name("Foo")
        .add_attribute_getter("a", |receiver: &Foo| receiver.a)
        // .add_method("b", |receiver: &Foo| oso::host::PolarResultIter(receiver.b()))
        .add_class_method("c", Foo::c)
        .add_method::<_, _, u32>("d", Foo::d)
        .add_method("e", Foo::e)
        // .add_method("f", |receiver: &Foo| oso::host::PolarResultIter(receiver.f()))
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

    common::setup();

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
    common::setup();

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
    common::setup();

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
fn test_enums() {
    common::setup();

    let mut test = OsoTest::new();

    // test an enum with no variants
    // this should simply not panic
    #[derive(Clone, PolarClass)]
    enum Foo {}

    test.oso.register_class(Foo::get_polar_class()).unwrap();

    // test an enum with variants
    #[derive(Clone, Debug, PartialEq, PolarClass)]
    enum Role {
        Admin,
        Member,
    }

    test.load_str(
        r#"
        is_admin(Role::Admin);
        is_member(Role::Member);"#,
    );

    test.oso
        .register_class(
            Role::get_polar_class_builder()
                .with_equality_check()
                .build(),
        )
        .unwrap();

    test.qvar_one(r#"is_admin(x)"#, "x", Role::Admin);
    test.qvar_one(r#"is_member(x)"#, "x", Role::Member);
}

#[test]
fn test_enums_and_structs() {
    common::setup();

    let mut test = OsoTest::new();
    test.load_str("allow(user: User, _action, _resource) if user.role = Role::Admin;");

    #[derive(Clone, Debug, PolarClass)]
    struct User {
        #[polar(attribute)]
        role: Role,
    }

    #[derive(Clone, Debug, PartialEq, PolarClass)]
    enum Role {
        Admin,
        Member,
    }

    test.oso.register_class(User::get_polar_class()).unwrap();

    test.oso
        .register_class(
            Role::get_polar_class_builder()
                .with_equality_check()
                .build(),
        )
        .unwrap();

    let admin = User { role: Role::Admin };

    let member = User { role: Role::Member };

    assert!(test.oso.is_allowed(admin, "read", "resource").unwrap());
    assert!(!test.oso.is_allowed(member, "read", "resource").unwrap());
}

#[test]
fn test_results_and_options() {
    common::setup();

    #[derive(PolarClass)]
    struct Foo;

    #[derive(Error, Debug)]
    #[error("Test error")]
    struct Error;

    impl Foo {
        fn new() -> Self {
            Self
        }
        #[allow(clippy::unnecessary_wraps)]
        fn ok(&self) -> Result<i32, Error> {
            Ok(1)
        }
        #[allow(clippy::unnecessary_wraps)]
        fn err(&self) -> Result<i32, Error> {
            Err(Error)
        }
        #[allow(clippy::unnecessary_wraps)]
        fn some(&self) -> Option<i32> {
            Some(1)
        }
        #[allow(clippy::unnecessary_wraps)]
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
    // TODO (dhatch): Assert type of error
    // TODO (dhatch): Check nested method error
    test.query_err("new Foo().err()");
    test.qvar_one(r#"new Foo().some() = x"#, "x", Some(1));
    test.qvar_one(r#"x in new Foo().some()"#, "x", 1);

    // test.qnull(r#"new Foo().none() and y = 1"#);
    test.qvar_one(r#"new Foo().none() = nil and y = 1"#, "y", 1);

    let results = test.query("x in new Foo().none()");
    assert!(results.is_empty());
}

// this functionality isn't very useful for rust as long as we
// only support nullary constructors ...
#[test]
fn test_unify_external_internal() {
    let mut test = OsoTest::new();
    test.qeval("new List() = []");
    test.qeval("new Dictionary() = {}");
    test.qeval("new String() = \"\"");
    test.qeval("new Integer() = 0");
    test.qeval("new Float() = 0.0");
    test.qeval("new Boolean() = false");
}

// TODO: dhatch see if there is a relevant test to port.
#[test]
fn test_unify_externals() {
    let mut test = OsoTest::new();

    #[derive(PartialEq, Clone, Debug)]
    struct Foo {
        x: i64,
    }

    impl PolarClass for Foo {}
    impl Foo {
        fn new(x: i64) -> Self {
            Self { x }
        }
    }

    let foo_class = ClassBuilder::with_constructor(Foo::new)
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

    let mut results = test.oso.query_rule("foos_equal", (a, b)).unwrap();
    results.next().expect("At least one result").unwrap();

    // Ensure that equality on a type that doesn't support it fails.
    struct Bar {
        x: i64,
    }

    impl PolarClass for Bar {}
    impl Bar {
        fn new(x: i64) -> Self {
            Self { x }
        }
    }

    let bar_class = ClassBuilder::with_constructor(Bar::new)
        .name("Bar")
        .add_attribute_getter("x", |this: &Bar| this.x)
        .build();

    test.oso.register_class(bar_class).unwrap();

    #[derive(PartialEq, Clone, Debug)]
    struct Baz {
        x: i64,
    }

    impl PolarClass for Baz {}
    impl Baz {
        fn new(x: i64) -> Self {
            Self { x }
        }
    }

    let baz_class = ClassBuilder::with_constructor(Baz::new)
        .name("Baz")
        .add_attribute_getter("x", |this: &Baz| this.x)
        .with_equality_check()
        .build();

    test.oso.register_class(baz_class).unwrap();
}

#[test]
fn test_values() {
    let _ = tracing_subscriber::fmt::try_init();

    #[derive(PolarClass)]
    struct Foo;

    impl Foo {
        fn new() -> Self {
            Self
        }

        fn one_two_three(&self) -> Vec<i32> {
            vec![1, 2, 3]
        }
    }

    let mut test = OsoTest::new();
    test.oso
        .register_class(
            Foo::get_polar_class_builder()
                .set_constructor(Foo::new)
                .add_iterator_method("one_two_three", Foo::one_two_three)
                .add_method("as_list", Foo::one_two_three)
                .build(),
        )
        .unwrap();

    let results: Vec<i32> = test.qvar("x in new Foo().one_two_three()", "x");
    assert!(results == vec![1, 2, 3]);
    let result: Vec<Vec<i32>> = test.qvar("new Foo().as_list() = x", "x");
    assert!(result == vec![vec![1, 2, 3]]);
}

#[test]
fn test_arg_number() {
    let _ = tracing_subscriber::fmt::try_init();
    #[derive(PolarClass)]
    struct Foo;

    impl Foo {
        fn three(&self, one: i32, two: i32, three: i32) -> i32 {
            one + two + three
        }

        fn many_method(
            &self,
            one: i32,
            two: i32,
            three: i32,
            four: i32,
            five: i32,
            six: i32,
            seven: i32,
        ) -> i32 {
            one + two + three + four + five + six + seven
        }

        fn many_class_method(
            one: i32,
            two: i32,
            three: i32,
            four: i32,
            five: i32,
            six: i32,
            seven: i32,
        ) -> i32 {
            one + two + three + four + five + six + seven
        }
    }

    let mut test = OsoTest::new();
    test.oso
        .register_class(
            Foo::get_polar_class_builder()
                .add_method("many_method", Foo::three)
                .add_method("many_method", Foo::many_method)
                .add_class_method("many_class", Foo::many_class_method)
                .build(),
        )
        .unwrap();
}

#[test]
fn test_without_registering() {
    let _ = tracing_subscriber::fmt::try_init();
    #[derive(Clone, PolarClass)]
    struct Foo {
        #[polar(attribute)]
        x: u32,
    }

    let mut test = OsoTest::new();
    test.oso.load_str("f(foo: Foo) if 1 = foo.x;").unwrap();
    test.oso
        .query_rule("f", (Foo { x: 1 },))
        .unwrap()
        .next()
        .unwrap()
        .unwrap();
}

#[test]
fn test_option() {
    let _ = tracing_subscriber::fmt::try_init();
    #[derive(Clone, Default, PolarClass)]
    struct Foo;

    impl Foo {
        #[allow(clippy::unnecessary_wraps)]
        fn get_some(&self) -> Option<i32> {
            Some(12)
        }
        #[allow(clippy::unnecessary_wraps)]
        fn get_none(&self) -> Option<i32> {
            None
        }
    }

    let mut test = OsoTest::new();
    test.oso
        .register_class(
            Foo::get_polar_class_builder()
                .set_constructor(Foo::default)
                .add_method("get_some", Foo::get_some)
                .add_method("get_none", Foo::get_none)
                .build(),
        )
        .unwrap();
    test.qvar_one("new Foo().get_some() = x", "x", Some(12i32));
    test.qvar_one("x in new Foo().get_some()", "x", 12i32);
    test.qvar_one("new Foo().get_none() = x", "x", Option::<i32>::None);
    test.qeval("12 in new Foo().get_some()");
    test.qeval("new Foo().get_none() = nil");
}

#[test]
fn test_is_subclass() {
    let _ = tracing_subscriber::fmt::try_init();
    #[derive(Clone, Default, PolarClass, PartialEq)]
    struct Foo;

    #[derive(Clone, Default, PolarClass)]
    struct Bar;

    let mut test = OsoTest::new();
    test.oso
        .register_class(
            Foo::get_polar_class_builder()
                .set_constructor(Foo::default)
                .with_equality_check()
                .build(),
        )
        .unwrap();
    test.oso.register_class(Bar::get_polar_class()).unwrap();
    test.qeval("x matches Foo and x matches Foo and x = new Foo()");
    // should fail by checking that Foo != Bar so not a subclass
    test.qnull("x matches Foo and x matches Bar");
}

#[cfg(feature = "uuid-06")]
#[test]
fn test_uuid_06() -> Result<(), Box<dyn std::error::Error>> {
    use uuid_06::Uuid;
    let mut test = OsoTest::new();
    test.oso.register_class(Uuid::get_polar_class())?;
    test.load_str("f(x: Uuid, y: Uuid) if x = y;");
    let (x, y) = (Uuid::nil(), Uuid::nil());
    test.oso.query_rule("f", (x, y))?.next().unwrap()?;
    Ok(())
}

#[cfg(feature = "uuid-07")]
#[test]
fn test_uuid_07() -> Result<(), Box<dyn std::error::Error>> {
    use uuid_07::Uuid;
    let mut test = OsoTest::new();
    test.oso.register_class(Uuid::get_polar_class())?;
    test.load_str("f(x: Uuid, y: Uuid) if x = y;");
    let (x, y) = (Uuid::nil(), Uuid::nil());
    test.oso.query_rule("f", (x, y))?.next().unwrap()?;
    Ok(())
}

#[cfg(feature = "uuid-10")]
#[test]
fn test_uuid_10() -> Result<(), Box<dyn std::error::Error>> {
    use uuid_10::Uuid;
    let mut test = OsoTest::new();
    test.oso.register_class(Uuid::get_polar_class())?;
    test.load_str("f(x: Uuid, y: Uuid) if x = y;");
    let (x, y) = (Uuid::nil(), Uuid::nil());
    test.oso.query_rule("f", (x, y))?.next().unwrap()?;
    Ok(())
}
