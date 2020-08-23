use js_sys::{Error, JsString, Map, Object, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn load_file_succeeds() {
    let polar = polar_wasm_api::Polar::wasm_new();
    let res = polar.wasm_load("x() if 1 == 1;\n", Some("foo.polar".to_owned()));
    assert!(matches!(res, Ok(())));
}

#[wasm_bindgen_test]
fn load_file_errors() {
    let polar = polar_wasm_api::Polar::wasm_new();
    let err = polar.wasm_load(";", None).unwrap_err();
    let err: Error = err.dyn_into().unwrap();
    assert_eq!(err.name(), "ParseError::UnrecognizedToken");
    assert_eq!(
        err.message(),
        "did not expect to find the token ';' at line 1, column 1"
    );
}

#[wasm_bindgen_test]
fn next_inline_query_succeeds() {
    let polar = polar_wasm_api::Polar::wasm_new();
    let res = polar.wasm_load("?= 1 = 1;", None);
    assert!(matches!(res, Ok(())));

    let mut query = polar.wasm_next_inline_query().unwrap();
    let event: Object = query.wasm_next_event().unwrap().dyn_into().unwrap();
    let event_kind: JsValue = "Result".into();
    let event_data = Reflect::get(&event, &event_kind).unwrap();
    let data_key: JsValue = "bindings".into();
    let bindings = Reflect::get(&event_data, &data_key).unwrap();
    assert_eq!(bindings.dyn_into::<Map>().unwrap().size(), 0);

    let event: JsString = query.wasm_next_event().unwrap().dyn_into().unwrap();
    assert_eq!(event, "Done");

    assert!(polar.wasm_next_inline_query().is_none());
}

#[wasm_bindgen_test]
fn next_inline_query_errors() {
    let polar = polar_wasm_api::Polar::wasm_new();
    let res = polar.wasm_load("?= 1 = 2;", None);
    assert!(matches!(res, Ok(())));
    let mut query = polar.wasm_next_inline_query().unwrap();
    let event: JsString = query.wasm_next_event().unwrap().dyn_into().unwrap();
    assert_eq!(event, "Done");
    assert!(polar.wasm_next_inline_query().is_none());
}

#[wasm_bindgen_test]
fn register_constant_succeeds() {
    let mut polar = polar_wasm_api::Polar::wasm_new();
    let res = polar.wasm_register_constant(
        "mathematics",
        r#"{"value":{"ExternalInstance":{"instance_id":1,"literal":null,"repr":null}}}"#,
    );
    assert!(matches!(res, Ok(())));
}

#[wasm_bindgen_test]
fn register_constant_errors() {
    let mut polar = polar_wasm_api::Polar::wasm_new();
    let err = polar.wasm_register_constant("mathematics", "").unwrap_err();
    let err: Error = err.dyn_into().unwrap();
    assert_eq!(err.name(), "RuntimeError::Serialization");
    assert_eq!(
        err.message(),
        "Serialization error: EOF while parsing a value at line 1 column 0"
    );
}

#[wasm_bindgen_test]
fn new_query_from_str_succeeds() {
    let polar = polar_wasm_api::Polar::wasm_new();
    let mut query = polar.wasm_new_query_from_str("x()").unwrap();
    let event: JsString = query.wasm_next_event().unwrap().dyn_into().unwrap();
    assert_eq!(event, "Done");
}

#[wasm_bindgen_test]
fn new_query_from_str_errors() {
    let polar = polar_wasm_api::Polar::wasm_new();
    let mut query = polar.wasm_new_query_from_str("x").unwrap();
    let err: Error = query.wasm_next_event().unwrap_err().dyn_into().unwrap();
    assert_eq!(err.name(), "RuntimeError::TypeError");
    assert_eq!(err.message(), "trace (most recent evaluation last):\n  in query at line 1, column 1\n    x\nType error: can't query for: x at line 1, column 1");
}

#[wasm_bindgen_test]
fn new_query_from_term_succeeds() {
    let polar = polar_wasm_api::Polar::wasm_new();
    let term = r#"{"value":{"Call":{"name":"x","args":[]}}}"#;
    let mut query = polar.wasm_new_query_from_term(term).unwrap();
    let event: JsString = query.wasm_next_event().unwrap().dyn_into().unwrap();
    assert_eq!(event, "Done");
}

#[wasm_bindgen_test]
fn new_query_from_term_errors() {
    let polar = polar_wasm_api::Polar::wasm_new();
    let res = polar.wasm_new_query_from_term("");
    if let Err(err) = res {
        let err: Error = err.dyn_into().unwrap();
        assert_eq!(err.name(), "RuntimeError::Serialization");
        assert_eq!(
            err.message(),
            "Serialization error: EOF while parsing a value at line 1 column 0"
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
