/// Common tests for all integrations.
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};

use oso::{Class, FromPolar, HostClass, Oso, OsoError, PolarClass, ToPolar, Value};
use oso_derive::*;
use polar_core::error as polar_error;

use maplit::hashmap;

mod common;

use common::OsoTest;

fn test_file_path() -> PathBuf {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = path.join(Path::new("tests/test_file.polar"));
    path
}

fn test_file_gx_path() -> PathBuf {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = path.join(Path::new("tests/test_file_gx.polar"));
    path
}

// EXTERNALS

#[derive(PolarClass, Debug, Clone, PartialEq)]
struct Widget {
    #[polar(attribute)]
    id: i64,
}

impl Widget {
    pub fn new(id: i64) -> Self {
        Self { id }
    }

    fn polar_class() -> Class {
        Widget::get_polar_class_builder()
            .name("Widget")
            .set_constructor(Self::new)
            .build()
    }
}

#[derive(PolarClass, Debug, Clone, PartialEq)]
struct Actor {
    #[polar(attribute)]
    name: String,
}

impl Actor {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn widget(&self) -> Widget {
        Widget::new(1)
    }

    pub fn widgets() {
        todo!("Iterator returning multiple choices not yet implemented.");
    }

    fn polar_class() -> Class {
        Actor::get_polar_class_builder()
            .name("Actor")
            .add_method("widget", Actor::widget)
            .build()
    }
}

fn test_oso() -> OsoTest {
    let mut test = OsoTest::new();
    test.oso.register_class(Widget::polar_class()).unwrap();
    test.oso.register_class(Actor::polar_class()).unwrap();

    test
}

#[test]
fn test_anything_works() -> oso::Result<()> {
    common::setup();
    let mut oso = Oso::new();
    oso.load_str("f(1);")?;

    let mut query = oso.query("f(x)")?;
    let next = query.next().unwrap()?;
    let x: i64 = next.get_typed("x")?;
    assert_eq!(x, 1);
    assert_eq!(
        next.keys().map(&str::to_owned).collect::<Vec<_>>(),
        vec!["x"]
    );

    Ok(())
}

#[test]
fn test_data_conversions_polar_values() -> oso::Result<()> {
    common::setup();

    let mut test_oso = OsoTest::new();

    // Converts Polar values into Rust values.
    test_oso.load_str(r#"f({x: [1, "two", true], y: {z: false}});"#);
    let mut query = test_oso.oso.query("f(x)")?;

    let x: HashMap<String, Value> = query.next().unwrap()?.get_typed("x")?;

    let v_x = x.get("x").unwrap();

    // TODO (dhatch): Type handling: Would be great to be able to get each index
    // out here dynamically, the same way we can with result set.
    if let Value::List(x_vec) = v_x {
        assert_eq!(
            query.from_polar::<i64>(&x_vec.get(0).unwrap().to_owned())?,
            1
        );
        assert_eq!(
            query.from_polar::<String>(&x_vec.get(1).unwrap().to_owned())?,
            String::from("two")
        );
        assert_eq!(
            query.from_polar::<bool>(&x_vec.get(2).unwrap().to_owned())?,
            true
        );
    } else {
        panic!("x not list.");
    }

    let v_y = x.get("y").unwrap();
    let y: HashMap<String, bool> = query.from_polar_value(v_y.to_owned())?;
    assert_eq!(y, hashmap! {String::from("z") => false});

    Ok(())
}

// TODO (dhatch): No predicate right now.
#[ignore]
#[test]
fn test_data_conversions_predicates() -> oso::Result<()> {
    common::setup();

    let mut test_oso = OsoTest::new();
    test_oso.load_str("f(x) if pred(1, 2);");

    todo!("No predicate in API");
}

#[test]
fn test_data_conversions_instances() {
    // TODO (dhatch): Ruby version of this test is not an integration test, not ported.
}

#[test]
fn test_data_conversions_externals() -> oso::Result<()> {
    common::setup();
    let mut oso = test_oso();

    let actor = Actor::new(String::from("sam"));
    let widget = Widget::new(1);

    oso.load_str("allow(actor, resource) if actor.widget().id = resource.id;");
    let query_results = oso
        .oso
        .query_rule(
            "allow",
            vec![&actor as &dyn ToPolar, &widget as &dyn ToPolar],
        )?
        .map(|r| r.unwrap())
        .collect::<Vec<_>>();

    assert_eq!(query_results.len(), 1);

    Ok(())
}

#[ignore]
#[test]
fn test_data_conversion_iterator_external_calls() {
    todo!("Unimplemented");
}

#[ignore]
#[test]
fn test_data_conversions_no_leak() {
    // TODO not integration test
    todo!("Unimplemented.");
}

#[test]
fn test_load_file_error_contains_filename() {
    common::setup();
    let mut oso = test_oso();

    let mut tempfile = tempfile::Builder::new()
        .suffix(".polar")
        .tempfile()
        .unwrap();
    let file = tempfile.as_file_mut();

    writeln!(file, ";").unwrap();
    file.sync_all().unwrap();

    let err = oso.oso.load_file(tempfile.path()).unwrap_err();
    if let OsoError::Polar(err) = err {
        assert_eq!(
            err.to_string(),
            format!(
                "did not expect to find the token ';' at line 1, column 1 in file {}",
                tempfile.path().to_string_lossy().into_owned()
            )
        );
    } else {
        panic!("Unexpected error type {:?}", err);
    }
}

#[test]
fn test_load_file_extension_check() {
    common::setup();

    let mut oso = test_oso();

    let err = oso.oso.load_file("not_polar_file.txt").unwrap_err();
    assert!(
        matches!(err, OsoError::IncorrectFileType { filename } if filename == "not_polar_file.txt")
    );
}

#[test]
fn test_load_file_nonexistent_file() {
    common::setup();

    let mut oso = test_oso();

    let err = oso.oso.load_file("not_a_file.polar").unwrap_err();
    assert!(matches!(err, OsoError::Io(_)));
}

#[test]
fn test_already_loaded_file_error() -> oso::Result<()> {
    common::setup();

    let mut oso = test_oso();
    let path = test_file_path();

    oso.oso.load_file(&path)?;
    let err = oso.oso.load_file(&path).unwrap_err();

    assert!(
        matches!(&err,
        OsoError::Polar(polar_error::PolarError {
            kind:
                polar_error::ErrorKind::Runtime(polar_error::RuntimeError::FileLoading { .. }),
            ..
        })
    if &err.to_string() == &format!("Problem loading file: File {} has already been loaded.", path.to_string_lossy())),
        "Error was {:?}",
        &err
    );

    Ok(())
}

#[test]
fn test_load_multiple_files() -> oso::Result<()> {
    common::setup();

    let mut oso = test_oso();
    let path = test_file_path();
    let path_gx = test_file_gx_path();

    oso.oso.load_file(path)?;
    oso.oso.load_file(path_gx)?;

    assert_eq!(oso.qvar::<i64>("f(x)", "x"), vec![1, 2, 3]);
    assert_eq!(oso.qvar::<i64>("g(x)", "x"), vec![1, 2, 3]);

    Ok(())
}

#[test]
fn test_clear() -> oso::Result<()> {
    common::setup();

    let mut oso = test_oso();
    oso.oso.load_file(test_file_path())?;

    assert_eq!(oso.qvar::<i64>("f(x)", "x"), vec![1, 2, 3]);
    oso.oso.clear();

    oso.qnull("f(x)");

    Ok(())
}

#[test]
fn test_basic_queries() -> oso::Result<()> {
    common::setup();

    let mut oso = test_oso();
    oso.load_str("f(1);");
    let results = oso.query("f(1)");

    assert_eq!(results.len(), 1);
    assert_eq!(
        results
            .get(0)
            .map(|r| r.keys().next().is_none())
            .unwrap_or_default(),
        true
    );

    Ok(())
}

// TODO unit test
//#[test]
//fn test_constructor_positional() -> oso::Result<()> {
//common::setup();

//let mut oso = test_oso();

//#[derive(PolarClass, Debug, Clone)]
//struct Foo {
//#[polar(attribute)]
//bar: i64,
//#[polar(attribute)]
//baz: i64,
//}

//impl Foo {
//pub fn new(bar: i64, baz: i64) -> Self {
//Self { bar, baz }
//}
//}

//let foo_class = Foo::get_polar_class_builder()
//.set_constructor(Foo::new)
//.name("Foo")
//.build();

//oso.oso.register_class(foo_class)?;

//Ok(());
//}

#[test]
fn test_register_constant() -> oso::Result<()> {
    common::setup();

    let mut oso = test_oso();

    let d = hashmap! {String::from("a") => 1};
    oso.oso.register_constant("d", &d)?;

    assert_eq!(oso.qvar::<i64>("d.a = x", "x"), vec![1]);

    Ok(())
}

#[ignore]
#[test]
fn test_host_method_string() {
    todo!();
}

#[ignore]
#[test]
fn test_host_method_integer() {
    todo!();
}

#[ignore]
#[test]
fn test_host_method_float() {
    todo!();
}

#[ignore]
#[test]
fn test_host_method_list() {
    todo!();
}

#[ignore]
#[test]
fn test_host_method_dict() {
    todo!();
}

// test_host_method_nil skipped. Covered by option tests.

#[test]
fn test_duplicate_register_class() -> oso::Result<()> {
    common::setup();

    let mut oso = test_oso();

    #[derive(PolarClass, Default, Debug, Clone)]
    struct Foo {};

    let foo_class = Foo::get_polar_class_builder().name("Foo").build();

    oso.oso.register_class(foo_class.clone())?;
    let err = oso.oso.register_class(foo_class).unwrap_err();
    assert!(matches!(err, OsoError::DuplicateClassError { name } if &name == "Foo"));

    Ok(())
}

// test_duplicate_register_class_alias skipped. Functionality covered above.

#[test]
fn test_register_class() -> oso::Result<()> {
    common::setup();

    let mut oso = test_oso();

    #[derive(PolarClass, Default, Debug, Clone)]
    struct Bar;

    impl Bar {
        pub fn y(&self) -> String {
            "y".to_owned()
        }
    }

    let bar_class = Bar::get_polar_class_builder()
        .name("Bar")
        .add_method("y", Bar::y)
        .build();

    #[derive(PolarClass, Debug, Clone)]
    struct Foo {
        #[polar(attribute)]
        a: String
    }

    impl Foo {
        pub fn new(a: String) -> Self {
            Self { a }
        }

        pub fn b(&self) -> Vec<String> {
            vec!["b".to_owned()]
        }

        pub fn c(&self) -> String {
            "c".to_owned()
        }

        pub fn d(&self, x: String) -> String {
            x
        }

        pub fn bar(&self) -> Bar {
            Bar::default()
        }

        pub fn e(&self) -> Vec<i64> {
            vec![1, 2, 3]
        }

        pub fn f(&self) -> Vec<Vec<i64>> {
            // NOTE: Slight different with ruby test.
            // Ruby tests with yielding multiple types, we
            // only yield one.
            vec![
                vec![1, 2, 3],
                vec![4, 5, 6]
            ]
        }

        pub fn g(&self) -> HashMap<String, String> {
            hashmap!{"hello".to_owned() => "world".to_owned()}
        }

        pub fn h(&self) -> bool {
            true
        }
    }

    let foo_class = Foo::get_polar_class_builder()
        .name("Foo")
        .set_constructor(|| Foo::new("A".to_owned()))
        .add_method("b", Foo::b)
        .add_method("c", Foo::c)
        .add_method("d", Foo::d)
        .add_method("bar", Foo::bar)
        .add_method("e", Foo::e)
        // TODO make this an iterator
        .add_method("f", Foo::f)
        .add_method("g", Foo::g)
        .add_method("h", Foo::h)
        .build();

    oso.oso.register_class(bar_class)?;
    oso.oso.register_class(foo_class)?;

    oso.qvar_one("new Foo().a = x", "x", String::from("A"));
    oso.query_err("new Foo().b = x");
    oso.qvar_one("new Foo().b() = x", "x", vec!["b".to_owned()]);
    oso.qvar_one("new Foo().c() = x", "x", "c".to_owned());
    oso.qvar_one("new Foo() = f and f.a = x", "x", "A".to_owned());
    oso.qvar_one("new Foo().bar().y() = x", "x", "y".to_owned());
    oso.qvar_one("new Foo().e() = x", "x", vec![1, 2, 3]);
    // TODO oso.qvar_one("new Foo().f() = x", "x", vec![1, 2, 3]);
    oso.qvar_one("new Foo().g().hello = x", "x", "world".to_owned());
    oso.qvar_one("new Foo().h() = x", "x", true);

    Ok(())
}

// test_class_inheritance skipped, no inheritance.

#[test]
fn test_animals() -> oso::Result<()> {
    common::setup();

    let mut oso = test_oso();

    #[derive(PolarClass, Debug, Clone, PartialEq)]
    struct Animal {
        #[polar(attribute)]
        family: String,
        #[polar(animal)]
        genus: String,
        #[polar(animal)]
        species: String
    }

    impl Animal {
        pub fn new(family: String, genus: String, species: String) -> Self {
            Self { family, genus, species }
        }
    }

    let animal_class = Animal::get_polar_class_builder()
        .name("Animal")
        .set_constructor(Animal::new)
        .with_equality_check()
        .build();

    oso.oso.register_class(animal_class)?;

    Ok(())
}
