/// Common tests for all integrations.
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};

use oso::{Class, FromPolar, Oso, OsoError, PolarClass, PolarValue};
use polar_core::error as polar_error;

use maplit::hashmap;

mod common;

use common::OsoTest;

fn test_file_path() -> PathBuf {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"));
    path.join(Path::new("tests/test_file.polar"))
}

fn test_file_gx_path() -> PathBuf {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"));
    path.join(Path::new("tests/test_file_gx.polar"))
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
struct User {
    #[polar(attribute)]
    name: String,
}

impl User {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn widget(&self) -> Widget {
        Widget::new(1)
    }

    #[allow(dead_code)]
    pub fn widgets() {
        todo!("Iterator returning multiple choices not yet implemented.");
    }

    fn polar_class() -> Class {
        User::get_polar_class_builder()
            .name("User")
            .add_method("widget", User::widget)
            .build()
    }
}

fn test_oso() -> OsoTest {
    let mut test = OsoTest::new();
    test.oso.register_class(Widget::polar_class()).unwrap();
    test.oso.register_class(User::polar_class()).unwrap();

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

    let x: HashMap<String, PolarValue> = query.next().unwrap()?.get_typed("x")?;

    let v_x = x.get("x").unwrap();

    // TODO (dhatch): Type handling: Would be great to be able to get each index
    // out here dynamically, the same way we can with result set.
    if let PolarValue::List(x_vec) = v_x {
        assert_eq!(i64::from_polar(x_vec.get(0).unwrap().to_owned())?, 1);
        assert_eq!(
            String::from_polar(x_vec.get(1).unwrap().to_owned())?,
            String::from("two")
        );
        assert!(bool::from_polar(x_vec.get(2).unwrap().to_owned())?);
    } else {
        panic!("x not list.");
    }

    let v_y = x.get("y").unwrap();
    let y: HashMap<String, bool> = HashMap::<String, bool>::from_polar(v_y.to_owned())?;
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

    let actor = User::new(String::from("sam"));
    let widget = Widget::new(1);

    oso.load_str("allow(actor, _action, resource) if actor.widget().id = resource.id;");
    let query_results = oso
        .oso
        .query_rule("allow", (actor, "read", widget))?
        .count();

    assert_eq!(query_results, 1);

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

    let err = oso.oso.load_files(vec![tempfile.path()]).unwrap_err();
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

    let err = oso.oso.load_files(vec!["not_polar_file.txt"]).unwrap_err();
    assert!(
        matches!(err, OsoError::IncorrectFileType { filename } if filename == "not_polar_file.txt")
    );
}

#[test]
fn test_load_file_nonexistent_file() {
    common::setup();

    let mut oso = test_oso();

    let err = oso.oso.load_files(vec!["not_a_file.polar"]).unwrap_err();
    assert!(matches!(err, OsoError::Io(_)));
}

#[test]
fn test_already_loaded_file_error() -> oso::Result<()> {
    common::setup();

    let mut oso = test_oso();
    let path = test_file_path();

    let err = oso.oso.load_files(vec![&path, &path]).unwrap_err();

    assert!(
        matches!(&err,
        OsoError::Polar(polar_error::PolarError {
            kind:
                polar_error::ErrorKind::Runtime(polar_error::RuntimeError::FileLoading { .. }),
            ..
        })
    if err.to_string() == format!("Problem loading file: File {} has already been loaded.", path.to_string_lossy())),
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

    oso.oso.load_files(vec![path, path_gx])?;

    assert_eq!(oso.qvar::<i64>("f(x)", "x"), vec![1, 2, 3]);
    assert_eq!(oso.qvar::<i64>("g(x)", "x"), vec![1, 2, 3]);

    Ok(())
}

#[test]
fn test_clear_rules() -> oso::Result<()> {
    common::setup();

    let mut oso = test_oso();
    oso.oso.load_files(vec![test_file_path()])?;
    assert_eq!(oso.qvar::<i64>("f(x)", "x"), vec![1, 2, 3]);

    #[derive(PolarClass, Default, Debug, Clone)]
    struct Foo;
    impl Foo {
        pub fn new() -> Self {
            Self {}
        }
    }
    let foo_class = Foo::get_polar_class_builder()
        .name("Foo")
        .set_constructor(Foo::new)
        .build();

    oso.oso.register_class(foo_class)?;

    assert!(matches!(oso.oso.clear_rules(), Ok(())));

    oso.qnull("f(x)");
    assert_eq!(oso.query("x = new Foo()").len(), 1);

    Ok(())
}

#[test]
fn test_basic_queries() {
    common::setup();

    let mut oso = test_oso();
    oso.load_str("f(1);");
    let results = oso.query("f(1)");

    assert_eq!(results.len(), 1);
    assert!(results
        .get(0)
        .map(|r| r.keys().next().is_none())
        .unwrap_or_default());
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
    oso.oso.register_constant(d, "d")?;

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
    struct Foo {}

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
        pub fn y(&self) -> &'static str {
            "y"
        }
    }

    let bar_class = Bar::get_polar_class_builder()
        .name("Bar")
        .add_method("y", Bar::y)
        .build();

    #[derive(PolarClass, Debug, Clone)]
    struct Foo {
        #[polar(attribute)]
        a: String,
    }

    impl Foo {
        pub fn new(a: String) -> Self {
            Self { a }
        }

        pub fn b(&self) -> Vec<&'static str> {
            vec!["b"]
        }

        pub fn c(&self) -> &'static str {
            "c"
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
            vec![vec![1, 2, 3], vec![4, 5, 6]]
        }

        pub fn g(&self) -> HashMap<&'static str, &'static str> {
            hashmap! {"hello" => "world"}
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
        #[polar(attribute)]
        genus: String,
        #[polar(attribute)]
        species: String,
    }

    impl Animal {
        pub fn new(family: String, genus: String, species: String) -> Self {
            Self {
                family,
                genus,
                species,
            }
        }
    }

    let animal_class = Animal::get_polar_class_builder()
        .name("Animal")
        .set_constructor(Animal::new)
        .with_equality_check()
        .build();

    oso.oso.register_class(animal_class)?;

    let wolf = r#"new Animal("canidae", "canis", "canis lupus")"#;
    let dog = r#"new Animal("canidae", "canis", "canis familiaris")"#;
    let canine = r#"new Animal("canidae", "canis", "")"#;
    let canid = r#"new Animal("canidae", "", "")"#;
    let animal = r#"new Animal("", "", "")"#;

    oso.load_str(
        r#"
      yup() if new Animal("steve", "", "") = new Animal("steve", "", "");
      nope() if new Animal("steve", "", "") = new Animal("gabe", "", "");
    "#,
    );

    oso.qeval("yup()");
    oso.qnull("nope()");

    oso.clear_rules();
    oso.load_str(
        r#"
      what_is(_: {genus: "canis"}, r) if r = "canine";
      what_is(_: {species: "canis lupus", genus: "canis"}, r) if r = "wolf";
      what_is(_: {species: "canis familiaris", genus: "canis"}, r) if r = "dog";
    "#,
    );

    assert_eq!(
        oso.qvar::<String>(&format!("what_is({}, r)", wolf), "r"),
        vec!["wolf".to_owned(), "canine".to_owned()]
    );
    assert_eq!(
        oso.qvar::<String>(&format!("what_is({}, r)", dog), "r"),
        vec!["dog".to_owned(), "canine".to_owned()]
    );
    assert_eq!(
        oso.qvar::<String>(&format!("what_is({}, r)", canine), "r"),
        vec!["canine".to_owned()]
    );

    oso.clear_rules();
    oso.load_str(
        r#"
          what_is_class(_: Animal{}, r) if r = "animal";
          what_is_class(_: Animal{genus: "canis"}, r) if r = "canine";
          what_is_class(_: Animal{family: "canidae"}, r) if r = "canid";
          what_is_class(_: Animal{species: "canis lupus", genus: "canis"}, r) if r = "wolf";
          what_is_class(_: Animal{species: "canis familiaris", genus: "canis"}, r) if r = "dog";
          what_is_class(_: Animal{species: s, genus: "canis"}, r) if r = s;
    "#,
    );

    assert_eq!(
        oso.qvar::<String>(&format!("what_is_class({}, r)", wolf), "r"),
        vec![
            "wolf".to_owned(),
            "canis lupus".to_owned(),
            "canine".to_owned(),
            "canid".to_owned(),
            "animal".to_owned()
        ]
    );

    assert_eq!(
        oso.qvar::<String>(&format!("what_is_class({}, r)", dog), "r"),
        vec![
            "dog".to_owned(),
            "canis familiaris".to_owned(),
            "canine".to_owned(),
            "canid".to_owned(),
            "animal".to_owned()
        ]
    );

    assert_eq!(
        oso.qvar::<String>(&format!("what_is_class({}, r)", canine), "r"),
        vec![
            "".to_owned(),
            "canine".to_owned(),
            "canid".to_owned(),
            "animal".to_owned()
        ]
    );

    assert_eq!(
        oso.qvar::<String>(&format!("what_is_class({}, r)", canid), "r"),
        vec!["canid".to_owned(), "animal".to_owned()]
    );

    assert_eq!(
        oso.qvar::<String>(&format!("what_is_class({}, r)", animal), "r"),
        vec!["animal".to_owned()]
    );

    oso.clear_rules();
    oso.load_str(
        r#"
      what_is_mix(_: Animal{}, r) if r = "animal_class";
      what_is_mix(_: Animal{genus: "canis"}, r) if r = "canine_class";
      what_is_mix(_: {genus: "canis"}, r) if r = "canine_dict";
      what_is_mix(_: Animal{family: "canidae"}, r) if r = "canid_class";
      what_is_mix(_: {species: "canis lupus", genus: "canis"}, r) if r = "wolf_dict";
      what_is_mix(_: {species: "canis familiaris", genus: "canis"}, r) if r = "dog_dict";
      what_is_mix(_: Animal{species: "canis lupus", genus: "canis"}, r) if r = "wolf_class";
      what_is_mix(_: Animal{species: "canis familiaris", genus: "canis"}, r) if r = "dog_class";
    "#,
    );

    let wolf_dict = r#"{species: "canis lupus", genus: "canis", family: "canidae"}"#;
    let dog_dict = r#"{species: "canis familiaris", genus: "canis", family: "canidae"}"#;
    let canine_dict = r#"{genus: "canis", family: "canidae"}"#;

    assert_eq!(
        oso.qvar::<String>(&format!("what_is_mix({}, r)", wolf), "r"),
        vec![
            "wolf_class".to_owned(),
            "canine_class".to_owned(),
            "canid_class".to_owned(),
            "animal_class".to_owned(),
            "wolf_dict".to_owned(),
            "canine_dict".to_owned()
        ]
    );

    assert_eq!(
        oso.qvar::<String>(&format!("what_is_mix({}, r)", dog), "r"),
        vec![
            "dog_class".to_owned(),
            "canine_class".to_owned(),
            "canid_class".to_owned(),
            "animal_class".to_owned(),
            "dog_dict".to_owned(),
            "canine_dict".to_owned()
        ]
    );

    assert_eq!(
        oso.qvar::<String>(&format!("what_is_mix({}, r)", canine), "r"),
        vec![
            "canine_class".to_owned(),
            "canid_class".to_owned(),
            "animal_class".to_owned(),
            "canine_dict".to_owned()
        ]
    );

    assert_eq!(
        oso.qvar::<String>(&format!("what_is_mix({}, r)", wolf_dict), "r"),
        vec!["wolf_dict".to_owned(), "canine_dict".to_owned()]
    );

    assert_eq!(
        oso.qvar::<String>(&format!("what_is_mix({}, r)", dog_dict), "r"),
        vec!["dog_dict".to_owned(), "canine_dict".to_owned()]
    );

    assert_eq!(
        oso.qvar::<String>(&format!("what_is_mix({}, r)", canine_dict), "r"),
        vec!["canine_dict".to_owned()]
    );

    Ok(())
}

#[test]
fn test_inline_queries() {
    common::setup();

    let mut oso = test_oso();

    // Success if all inlines succeed.
    oso.load_str("f(1); f(2); ?= f(1); ?= not f(3);");

    // Fails if inline fails.
    oso.oso.load_str("g(1); ?= g(2);").unwrap_err();
}

// Skipped parse error tests.

#[test]
fn test_predicate_return_list() {
    common::setup();

    #[derive(PolarClass, Debug, Clone)]
    struct User;

    impl User {
        pub fn new() -> Self {
            Self
        }

        pub fn groups(&self) -> Vec<String> {
            vec![
                "engineering".to_owned(),
                "social".to_owned(),
                "admin".to_owned(),
            ]
        }
    }

    let actor_class = User::get_polar_class_builder()
        .name("UserTwo")
        .add_method("groups", User::groups)
        .build();

    let mut oso = test_oso();
    oso.load_str(r#"allow(actor: UserTwo, "join", "party") if "social" in actor.groups();"#);
    oso.oso.register_class(actor_class).unwrap();

    let mut query = oso
        .oso
        .query_rule("allow", (User::new(), "join", "party"))
        .unwrap();

    let result = query.next().unwrap().unwrap();
    assert_eq!(result.keys().count(), 0);
}

// TODO (dhatch): API not great.

#[test]
fn test_variables_as_arguments() -> oso::Result<()> {
    common::setup();

    let mut oso = test_oso();

    oso.oso.load_files(vec![test_file_path()])?;

    let query = oso
        .oso
        .query_rule("f", (PolarValue::Variable("a".to_owned()),))?;

    let a_var = query
        .map(|r| r.unwrap().get_typed::<i64>("a").unwrap())
        .collect::<Vec<_>>();
    assert_eq!(a_var, vec![1, 2, 3]);

    Ok(())
}

// Skipped test_stack_trace, this is functionality that should be tested in core.
// TODO ^

#[test]
fn test_lookup_runtime_error() {
    common::setup();

    let mut oso = test_oso();
    oso.query(r#"new Widget(1) = {bar: "bar"}"#);
    oso.query_err(r#"new Widget(1).bar = "bar""#);
}

#[test]
fn test_returns_unbound_variable() -> oso::Result<()> {
    common::setup();

    let mut oso = test_oso();
    oso.load_str("rule(_, y) if y = 1;");

    let first = oso.query("rule(x, y)").pop().unwrap();

    assert_eq!(first.get_typed::<i64>("y")?, 1);
    assert!(matches!(first.get_typed("x")?, PolarValue::Variable(_)));

    Ok(())
}

#[test]
fn test_nan_inf() -> oso::Result<()> {
    common::setup();

    let mut oso = test_oso();
    oso.oso.register_constant(std::f64::INFINITY, "inf")?;
    oso.oso
        .register_constant(std::f64::NEG_INFINITY, "neg_inf")?;
    oso.oso.register_constant(std::f64::NAN, "nan")?;

    let x = oso.qvar::<f64>("x = nan", "x").pop().unwrap();
    assert!(x.is_nan());
    oso.qnull("nan = nan");

    assert!(oso.qvar::<f64>("x = inf", "x").pop().unwrap().is_infinite());
    assert!(oso.query("inf = inf").pop().is_some());

    oso.qvar_one("x = neg_inf", "x", std::f64::NEG_INFINITY);
    assert!(oso.query("neg_inf = neg_inf").pop().is_some());

    Ok(())
}

#[test]
fn test_iterators() -> oso::Result<()> {
    common::setup();
    #[derive(Default, PolarClass)]
    struct Foo {}

    #[derive(Clone, PolarClass)]
    struct Bar(Vec<u32>);

    impl IntoIterator for Bar {
        type Item = u32;
        type IntoIter = std::vec::IntoIter<u32>;

        fn into_iter(self) -> Self::IntoIter {
            self.0.into_iter()
        }
    }

    impl Bar {
        fn new(list: Vec<u32>) -> Self {
            Self(list)
        }
        fn sum(&self) -> u32 {
            self.0.iter().sum()
        }
    }

    let mut oso = test_oso();
    oso.oso.register_class(
        Foo::get_polar_class_builder()
            .set_constructor(Foo::default)
            .build(),
    )?;
    oso.oso.register_class(
        Bar::get_polar_class_builder()
            .set_constructor(Bar::new)
            .add_method("sum", Bar::sum)
            .with_iter()
            .build(),
    )?;

    assert_eq!(
        oso.query_err("x in new Foo()"),
        "Unsupported operation in for type Foo."
    );
    assert_eq!(
        oso.qvar::<u32>("x in new Bar([1, 2, 3])", "x"),
        vec![1, 2, 3]
    );
    oso.qvar_one("x = new Bar([1, 2, 3]).sum()", "x", 6u32);

    Ok(())
}

#[test]
fn test_nil() {
    common::setup();

    let mut oso = test_oso();
    oso.load_str("null(nil);");

    oso.qvar_one("null(x)", "x", Option::<PolarValue>::None);
    assert_eq!(
        oso.oso
            .query_rule("null", (Option::<PolarValue>::None,))
            .unwrap()
            .count(),
        1
    );
    assert_eq!(
        oso.oso
            .query_rule("null", (Vec::<PolarValue>::new(),))
            .unwrap()
            .count(),
        0
    );
    oso.qeval("nil.is_none()");
    oso.qnull("x in nil");
}

#[test]
fn test_expression_error() {
    common::setup();

    let mut oso = test_oso();
    oso.load_str("f(x) if x > 2;");

    let err = oso.query_err("f(x)");
    assert!(err.contains("unbound"));
}

#[test]
fn test_rule_types() {
    common::setup();
    let mut oso = test_oso();
    let mut policy = r#"type is_actor(_actor: User);
                        is_actor(_actor: User);"#
        .to_owned();
    oso.load_str(&policy);
    oso.clear_rules();

    policy += "is_actor(_actor: Widget);";
    let err = oso
        .oso
        .load_str(&policy)
        .expect_err("Expected validation error");

    assert!(matches!(
        &err,
        OsoError::Polar(polar_error::PolarError {
            kind: polar_error::ErrorKind::Validation(
                polar_error::ValidationError::InvalidRule { .. }
            ),
            ..
        })
    ));
}
