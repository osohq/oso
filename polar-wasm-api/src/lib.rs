mod errors;

use polar_core::{polar, types};
use wasm_bindgen::prelude::*;

use errors::{serde_serialization_error, serialization_error, Error};

// TODO(gj): figure out how to handle Rust panics in wasm.

type JsResult<T> = Result<T, JsValue>;

#[wasm_bindgen]
pub struct Polar(polar::Polar);

#[wasm_bindgen]
pub struct Query(polar::Query);

#[wasm_bindgen]
impl Polar {
    #[wasm_bindgen(constructor)]
    pub fn wasm_new() -> Self {
        Self(polar::Polar::new(None))
    }

    #[wasm_bindgen(js_class = Polar, js_name = loadFile)]
    pub fn wasm_load_file(&self, src: &str, filename: Option<String>) -> JsResult<()> {
        self.0
            .load_file(src, filename)
            .map_err(Error::from)
            .map_err(Error::into)
    }

    #[wasm_bindgen(js_class = Polar, js_name = registerConstant)]
    pub fn wasm_register_constant(&mut self, name: &str, value: &str) -> JsResult<()> {
        match serde_json::from_str(value) {
            Ok(term) => self.0.register_constant(types::Symbol::new(name), term),
            Err(e) => return Err(serde_serialization_error(e)),
        }
        Ok(())
    }

    #[wasm_bindgen(js_class = Polar, js_name = nextInlineQuery)]
    pub fn wasm_next_inline_query(&self) -> Option<Query> {
        self.0.next_inline_query(false).map(Query)
    }

    #[wasm_bindgen(js_class = Polar, js_name = newQueryFromStr)]
    pub fn wasm_new_query_from_str(&self, src: &str) -> JsResult<Query> {
        self.0
            .new_query(src, false)
            .map(Query)
            .map_err(Error::from)
            .map_err(Error::into)
    }

    #[wasm_bindgen(js_class = Polar, js_name = newQueryFromTerm)]
    pub fn wasm_new_query_from_term(&self, value: &str) -> JsResult<Query> {
        serde_json::from_str(value)
            .map(|term| Query(self.0.new_query_from_term(term, false)))
            .map_err(serde_serialization_error)
    }

    #[wasm_bindgen(js_class = Polar, js_name = newId)]
    pub fn wasm_get_external_id(&self) -> u64 {
        self.0.get_external_id()
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
    pub fn wasm_call_result(&mut self, call_id: u64, value: Option<String>) -> JsResult<()> {
        let term: Option<types::Term> = if let Some(value) = value {
            match serde_json::from_str(&value) {
                Ok(term) => Some(term),
                Err(e) => return Err(serde_serialization_error(e)),
            }
        } else {
            None
        };
        self.0
            .call_result(call_id, term)
            .map_err(Error::from)
            .map_err(Error::into)
    }

    #[wasm_bindgen(js_class = Query, js_name = questionResult)]
    pub fn wasm_question_result(&mut self, call_id: u64, result: bool) {
        self.0.question_result(call_id, result)
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
}
