use js_sys::Error;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn polar_load_file_succeeds() {
    let polar = polar_wasm_api::Polar::wasm_new();
    let res = polar.wasm_load_file("x() if 1 == 1;\n", Some("foo.polar".to_owned()));
    assert!(matches!(res, Ok(())));
}

#[wasm_bindgen_test]
fn polar_load_file_errors() {
    let polar = polar_wasm_api::Polar::wasm_new();
    match polar.wasm_load_file(";", None) {
        Err(e) => {
            if let Some(e) = &e.dyn_ref::<Error>() {
                assert_eq!(e.name(), "ParseError::UnrecognizedToken");
                assert_eq!(
                    e.message(),
                    "did not expect to find the token ';' at line 1, column 1"
                );
            } else {
                panic!();
            }
        }
        _ => panic!(),
    }
}

#[wasm_bindgen_test]
fn polar_register_constant_succeeds() {
    let mut polar = polar_wasm_api::Polar::wasm_new();
    let res = polar.wasm_register_constant(
        "mathematics",
        r#"{"value":{"ExternalInstance":{"instance_id":1,"literal":null,"repr":null}}}"#,
    );
    assert!(matches!(res, Ok(())));
}

#[wasm_bindgen_test]
fn polar_register_constant_errors() {
    let mut polar = polar_wasm_api::Polar::wasm_new();
    match polar.wasm_register_constant("mathematics", "") {
        Err(e) => {
            if let Some(e) = &e.dyn_ref::<Error>() {
                assert_eq!(e.name(), "RuntimeError::Serialization");
                assert_eq!(
                    e.message(),
                    "Serialization error: EOF while parsing a value at line 1 column 0"
                );
            } else {
                panic!();
            }
        }
        _ => panic!(),
    }
}
