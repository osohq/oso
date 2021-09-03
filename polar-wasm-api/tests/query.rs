use js_sys::{Error, JsString, Map, Object, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::*;

use polar_core::sources::Source;

#[wasm_bindgen_test]
#[allow(clippy::float_cmp)]
fn call_result_succeeds() {
    let mut polar = polar_wasm_api::Polar::wasm_new();
    polar
        .wasm_register_constant(
            "y",
            r#"{"value":{"ExternalInstance":{"instance_id":1,"literal":null,"repr":null}}}"#,
        )
        .unwrap();
    let source = Source {
        src: "x() if y.z;".to_owned(),
        filename: None,
    };
    let sources: JsValue = serde_wasm_bindgen::to_value(&vec![source]).unwrap();
    polar.wasm_load(sources).unwrap();
    let mut query = polar.wasm_new_query_from_str("x()").unwrap();
    let event: Object = query.wasm_next_event().unwrap().dyn_into().unwrap();
    let event_kind: JsValue = "ExternalCall".into();
    let event_data = Reflect::get(&event, &event_kind).unwrap();
    let event_data: Object = event_data.dyn_into().unwrap();
    let event_field: JsValue = "call_id".into();
    let call_id = Reflect::get(&event_data, &event_field).unwrap();
    assert_eq!(call_id, 3.0);

    let call_result = Some(r#"{"value":{"Boolean":true}}"#.to_string());
    query.wasm_call_result(3.0, call_result).unwrap();

    let event: Object = query.wasm_next_event().unwrap().dyn_into().unwrap();
    let event_kind: JsValue = "Result".into();
    let event_data = Reflect::get(&event, &event_kind).unwrap();
    let data_key: JsValue = "bindings".into();
    let bindings = Reflect::get(&event_data, &data_key).unwrap();
    assert_eq!(bindings.dyn_into::<Map>().unwrap().size(), 0);

    query.wasm_call_result(3.0, None).unwrap();

    let event: Object = query.wasm_next_event().unwrap().dyn_into().unwrap();
    let event_kind: JsValue = "Done".into();
    assert!(Reflect::get(&event, &event_kind).is_ok())
}

#[wasm_bindgen_test]
#[allow(clippy::float_cmp)]
fn app_error_succeeds() {
    let mut polar = polar_wasm_api::Polar::wasm_new();
    polar
        .wasm_register_constant(
            "y",
            r#"{"value":{"ExternalInstance":{"instance_id":1,"literal":null,"repr":null}}}"#,
        )
        .unwrap();
    let source = Source {
        src: "x() if y.z;".to_owned(),
        filename: None,
    };
    let sources: JsValue = serde_wasm_bindgen::to_value(&vec![source]).unwrap();
    polar.wasm_load(sources).unwrap();
    let mut query = polar.wasm_new_query_from_str("x()").unwrap();
    let event: Object = query.wasm_next_event().unwrap().dyn_into().unwrap();
    let event_kind: JsValue = "ExternalCall".into();
    let event_data = Reflect::get(&event, &event_kind).unwrap();
    let event_data: Object = event_data.dyn_into().unwrap();
    let event_field: JsValue = "call_id".into();
    let call_id = Reflect::get(&event_data, &event_field).unwrap();
    assert_eq!(call_id, 3.0);

    let msg = "doin' the hokey-pokey";
    query.wasm_app_error(msg).unwrap();

    let err: Error = query.wasm_next_event().unwrap_err().dyn_into().unwrap();
    assert_eq!(err.name(), "RuntimeError::Application");
    assert!(err.message().includes(msg, 0));
}

#[wasm_bindgen_test]
fn debug_command_succeeds() {
    let polar = polar_wasm_api::Polar::wasm_new();
    let mut query = polar.wasm_new_query_from_str("x()").unwrap();
    query.wasm_debug_command("h").unwrap();
    let event: Object = query.wasm_next_event().unwrap().dyn_into().unwrap();
    let event_kind: JsValue = "Debug".into();
    let event_data = Reflect::get(&event, &event_kind).unwrap();
    let event_data: Object = event_data.dyn_into().unwrap();
    let event_field: JsValue = "message".into();
    let msg = Reflect::get(&event_data, &event_field).unwrap();
    let msg: JsString = msg.dyn_into().unwrap();
    assert!(msg.includes("Debugger Commands", 0));
}
