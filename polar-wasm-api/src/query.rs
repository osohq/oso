use polar_core::{polar, terms::Term};
use wasm_bindgen::prelude::*;

use crate::errors::{serde_serialization_error, serialization_error, Error};
use crate::JsResult;

#[wasm_bindgen]
pub struct Query(polar::Query);

impl From<polar::Query> for Query {
    fn from(q: polar::Query) -> Self {
        Self(q)
    }
}

#[wasm_bindgen]
impl Query {
    #[wasm_bindgen(js_class = Query, js_name = nextEvent)]
    pub fn wasm_next_event(&mut self) -> JsResult<JsValue> {
        self.0
            .next_event()
            .map_err(Error::from)
            .map_err(Error::into)
            .and_then(|event| {
                serde_wasm_bindgen::to_value(&event).map_err(|e| serialization_error(e.to_string()))
            })
    }

    #[wasm_bindgen(js_class = Query, js_name = callResult)]
    pub fn wasm_call_result(&mut self, call_id: f64, value: Option<String>) -> JsResult<()> {
        let term: Option<Term> = if let Some(value) = value {
            match serde_json::from_str(&value) {
                Ok(term) => Some(term),
                Err(e) => return Err(serde_serialization_error(e)),
            }
        } else {
            None
        };
        self.0
            .call_result(call_id as u64, term)
            .map_err(Error::from)
            .map_err(Error::into)
    }

    #[wasm_bindgen(js_class = Query, js_name = questionResult)]
    pub fn wasm_question_result(&mut self, call_id: f64, result: bool) {
        self.0.question_result(call_id as u64, result)
    }

    #[wasm_bindgen(js_class = Query, js_name = debugCommand)]
    pub fn wasm_debug_command(&mut self, command: &str) -> JsResult<()> {
        self.0
            .debug_command(command)
            .map_err(Error::from)
            .map_err(Error::into)
    }

    #[wasm_bindgen(js_class = Query, js_name = appError)]
    pub fn wasm_app_error(&mut self, msg: &str) {
        self.0.application_error(msg.to_owned())
    }

    #[wasm_bindgen(js_class = Query, js_name = nextMessage)]
    pub fn wasm_next_message(&self) -> JsResult<JsValue> {
        let message = self.0.next_message();
        serde_wasm_bindgen::to_value(&message).map_err(|e| serialization_error(e.to_string()))
    }

    #[wasm_bindgen(js_class = Query, js_name = source)]
    pub fn wasm_source(&self) -> JsResult<JsValue> {
        let source = self.0.source_info();
        serde_wasm_bindgen::to_value(&source).map_err(|e| serialization_error(e.to_string()))
    }
}
