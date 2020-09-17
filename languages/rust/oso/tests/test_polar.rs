/// Common tests for all integrations.
use std::collections::HashMap;
use std::io::Write;

use oso::{Class, FromPolar, HostClass, Oso, PolarClass, ToPolar, Value, OsoError};
use oso_derive::*;

use maplit::hashmap;

mod common;

use common::OsoTest;

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
