/// Common tests for all integrations.
use std::collections::HashMap;

use oso::{Class, FromPolar, HostClass, Oso, PolarClass, ToPolar, Value};
use oso_derive::*;

use maplit::hashmap;

mod common;

use common::OsoTest;

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
    assert_eq!(y, hashmap!{String::from("z") => false});

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
fn test_data_conversions_externals() {
    // TODO
}

#[test]
fn test_data_conversion_iterator_external_calls() {
    // TODO
}

#[test]
fn test_data_conversions_no_leak() {
    // TODO not integration test
}
