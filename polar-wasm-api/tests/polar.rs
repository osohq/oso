use js_sys::{Boolean, Error, Map, Object, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::*;

use polar_core::sources::Source;
use polar_core::terms::*;

// TODO(gj): figure out how to define shared test helpers instead of duplicating these in
// tests/polar.rs & tests/query.rs.
fn is_done_event(event: Object) -> bool {
    let key: JsValue = "Done".into();
    let value = Reflect::get(&event, &key).unwrap();
    let key: JsValue = "result".into();
    let value = Reflect::get(&value, &key).unwrap();
    value.dyn_into::<Boolean>().is_ok()
}

fn is_result_event(event: Object) -> bool {
    let key: JsValue = "Result".into();
    let value = Reflect::get(&event, &key).unwrap();
    let key: JsValue = "bindings".into();
    let value = Reflect::get(&value, &key).unwrap();
    let value = value.dyn_into::<Map>().unwrap();
    value.size() == 0
}

#[wasm_bindgen_test]
fn load_file_succeeds() {
    let polar = polar_wasm_api::Polar::wasm_new();
    let source = Source::new_with_name("foo.polar", "x() if 1 == 1;\n");
    let sources: JsValue = serde_wasm_bindgen::to_value(&vec![source]).unwrap();
    let res = polar.wasm_load(sources);
    assert!(matches!(res, Ok(())));
}

#[wasm_bindgen_test]
fn load_file_errors() {
    let polar = polar_wasm_api::Polar::wasm_new();
    let source = Source::new(";");
    let sources: JsValue = serde_wasm_bindgen::to_value(&vec![source]).unwrap();
    let err = polar.wasm_load(sources).unwrap_err();
    let err: Error = err.dyn_into().unwrap();
    assert_eq!(err.name(), "ParseError::UnrecognizedToken");
    assert!(err.message().starts_with(
        "did not expect to find the token ';' at line 1, column 1",
        0
    ));
}

#[wasm_bindgen_test]
fn next_inline_query_succeeds() {
    let polar = polar_wasm_api::Polar::wasm_new();
    let source = Source::new("?= 1 = 1;");
    let sources: JsValue = serde_wasm_bindgen::to_value(&vec![source]).unwrap();
    let res = polar.wasm_load(sources);
    assert!(matches!(res, Ok(())));

    let mut query = polar.wasm_next_inline_query().unwrap();
    let event: Object = query.wasm_next_event().unwrap().dyn_into().unwrap();
    assert!(is_result_event(event));

    let event: Object = query.wasm_next_event().unwrap().dyn_into().unwrap();
    assert!(is_done_event(event));

    assert!(polar.wasm_next_inline_query().is_none());
}

#[wasm_bindgen_test]
fn next_inline_query_errors() {
    let polar = polar_wasm_api::Polar::wasm_new();
    let source = Source::new("?= 1 = 2;");
    let sources: JsValue = serde_wasm_bindgen::to_value(&vec![source]).unwrap();
    let res = polar.wasm_load(sources);
    assert!(matches!(res, Ok(())));
    let mut query = polar.wasm_next_inline_query().unwrap();

    let event: Object = query.wasm_next_event().unwrap().dyn_into().unwrap();
    assert!(is_done_event(event));

    assert!(polar.wasm_next_inline_query().is_none());
}

#[wasm_bindgen_test]
fn register_constant_succeeds() {
    let mut polar = polar_wasm_api::Polar::wasm_new();
    let res = polar.wasm_register_constant(
        "mathematics",
        serde_wasm_bindgen::to_value(&Term::from(Value::ExternalInstance(ExternalInstance {
            instance_id: 1,
            constructor: None,
            repr: None,
            class_repr: None,
        })))
        .unwrap(),
    );
    assert!(matches!(res, Ok(())));
}

#[wasm_bindgen_test]
fn new_query_from_str_succeeds() {
    let polar = polar_wasm_api::Polar::wasm_new();
    let source = Source::new("x(1);");
    let sources: JsValue = serde_wasm_bindgen::to_value(&vec![source]).unwrap();
    polar.wasm_load(sources).unwrap();

    let mut query = polar.wasm_new_query_from_str("x(2)").unwrap();
    let event: Object = query.wasm_next_event().unwrap().dyn_into().unwrap();
    assert!(is_done_event(event));

    let mut query = polar.wasm_new_query_from_str("x(1)").unwrap();
    let event: Object = query.wasm_next_event().unwrap().dyn_into().unwrap();
    assert!(is_result_event(event));
}

#[wasm_bindgen_test]
fn new_query_from_str_errors() {
    let polar = polar_wasm_api::Polar::wasm_new();
    let mut query = polar.wasm_new_query_from_str("[]").unwrap();
    let err: Error = query.wasm_next_event().unwrap_err().dyn_into().unwrap();
    assert_eq!(err.name(), "RuntimeError::TypeError");
    assert!(err.message().starts_with(
        r#"trace (most recent evaluation last):
  000: []
    in query at line 1, column 1

Type error: [] isn't something that is true or false so can't be a condition at line 1, column 1"#,
        0
    ));
}

#[wasm_bindgen_test]
fn new_query_from_term_succeeds() {
    let polar = polar_wasm_api::Polar::wasm_new();
    let source = Source::new("x(1);");
    let sources: JsValue = serde_wasm_bindgen::to_value(&vec![source]).unwrap();
    polar.wasm_load(sources).unwrap();

    let term = Term::from(Value::Call(Call {
        name: Symbol("x".into()),
        args: vec![Term::from(2)],
        kwargs: None,
    }));
    let term = serde_wasm_bindgen::to_value(&term).unwrap();
    let mut query = polar.wasm_new_query_from_term(term).unwrap();
    let event: Object = query.wasm_next_event().unwrap().dyn_into().unwrap();
    assert!(is_done_event(event));

    let term = Term::from(Value::Call(Call {
        name: Symbol("x".into()),
        args: vec![Term::from(1)],
        kwargs: None,
    }));
    let term = serde_wasm_bindgen::to_value(&term).unwrap();
    let mut query = polar.wasm_new_query_from_term(term).unwrap();
    let event: Object = query.wasm_next_event().unwrap().dyn_into().unwrap();
    assert!(is_result_event(event));
}

#[wasm_bindgen_test]
fn new_query_from_term_errors() {
    let polar = polar_wasm_api::Polar::wasm_new();
    let res = polar.wasm_new_query_from_term("".into());
    if let Err(err) = res {
        let err: Error = err.dyn_into().unwrap();
        assert_eq!(err.name(), "Error");
        assert_eq!(
            err.message(),
            "invalid type: string \"\", expected struct Term",
        );
    } else {
        panic!();
    }
}

#[wasm_bindgen_test]
#[allow(clippy::float_cmp)]
fn get_external_id_succeeds() {
    let polar = polar_wasm_api::Polar::wasm_new();
    assert_eq!(polar.wasm_get_external_id(), 1.0);
    assert_eq!(polar.wasm_get_external_id(), 2.0);
}
