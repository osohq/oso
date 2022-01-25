use js_sys::{Boolean, Error, JsString, Map, Object, Reflect};
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
#[allow(clippy::float_cmp)]
fn call_result_succeeds() {
    let mut polar = polar_wasm_api::Polar::wasm_new();
    let term = Term::from(Value::ExternalInstance(ExternalInstance {
        instance_id: 1,
        constructor: None,
        repr: None,
        class_repr: None,
        class_id: None,
    }));
    let term = serde_wasm_bindgen::to_value(&term).unwrap();
    polar.wasm_register_constant("y", term).unwrap();
    let source = Source::new("x() if y.z;");
    let sources: JsValue = serde_wasm_bindgen::to_value(&vec![source]).unwrap();
    polar.wasm_load(sources).unwrap();
    let mut query = polar.wasm_new_query_from_str("x()").unwrap();
    let event: Object = query.wasm_next_event().unwrap().dyn_into().unwrap();
    let event_kind: JsValue = "ExternalCall".into();
    let event_data = Reflect::get(&event, &event_kind).unwrap();
    let event_data: Object = event_data.dyn_into().unwrap();
    let event_field: JsValue = "call_id".into();
    let call_id = Reflect::get(&event_data, &event_field).unwrap();
    assert_eq!(call_id, 1);

    let call_result = serde_wasm_bindgen::to_value(&Term::from(Value::Boolean(true))).unwrap();
    query.wasm_call_result(1.0, call_result).unwrap();

    let event: Object = query.wasm_next_event().unwrap().dyn_into().unwrap();
    assert!(is_result_event(event));

    query.wasm_call_result(1.0, JsValue::undefined()).unwrap();

    let event: Object = query.wasm_next_event().unwrap().dyn_into().unwrap();
    assert!(is_done_event(event));
}

#[wasm_bindgen_test]
fn app_error_succeeds() {
    let mut polar = polar_wasm_api::Polar::wasm_new();
    let term = Term::from(Value::ExternalInstance(ExternalInstance {
        instance_id: 1,
        constructor: None,
        repr: None,
        class_repr: None,
        class_id: None,
    }));
    let term = serde_wasm_bindgen::to_value(&term).unwrap();
    polar.wasm_register_constant("y", term).unwrap();
    let source = Source::new("x() if y.z;");
    let sources: JsValue = serde_wasm_bindgen::to_value(&vec![source]).unwrap();
    polar.wasm_load(sources).unwrap();
    let mut query = polar.wasm_new_query_from_str("x()").unwrap();
    let event: Object = query.wasm_next_event().unwrap().dyn_into().unwrap();
    let event_kind: JsValue = "ExternalCall".into();
    let event_data = Reflect::get(&event, &event_kind).unwrap();
    let event_data: Object = event_data.dyn_into().unwrap();
    let event_field: JsValue = "call_id".into();
    let call_id = Reflect::get(&event_data, &event_field).unwrap();
    assert_eq!(call_id, 1);

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
