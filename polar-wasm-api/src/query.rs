use polar_core::{query, terms::Symbol};
use wasm_bindgen::prelude::*;

use crate::errors::{serialization_error, Error};
use crate::JsResult;

#[wasm_bindgen]
pub struct Query(query::Query);

impl From<query::Query> for Query {
    fn from(q: query::Query) -> Self {
        Self(q)
    }
}

#[allow(unused_variables)]
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
    pub fn wasm_call_result(&mut self, call_id: f64, term: JsValue) -> JsResult<()> {
        let term = serde_wasm_bindgen::from_value(term)?;
        self.0
            .call_result(call_id as u64, term)
            .map_err(Error::from)
            .map_err(Error::into)
    }

    #[wasm_bindgen(js_class = Query, js_name = questionResult)]
    pub fn wasm_question_result(&mut self, call_id: f64, result: bool) -> JsResult<()> {
        self.0
            .question_result(call_id as u64, result)
            .map_err(Error::from)
            .map_err(Error::into)
    }

    #[wasm_bindgen(js_class = Query, js_name = debugCommand)]
    pub fn wasm_debug_command(&mut self, command: &str) -> JsResult<()> {
        self.0
            .debug_command(command)
            .map_err(Error::from)
            .map_err(Error::into)
    }

    #[wasm_bindgen(js_class = Query, js_name = appError)]
    pub fn wasm_app_error(&mut self, msg: &str) -> JsResult<()> {
        self.0
            .application_error(msg.to_owned())
            .map_err(Error::from)
            .map_err(Error::into)
    }

    #[wasm_bindgen(js_class = Query, js_name = nextMessage)]
    pub fn wasm_next_message(&self) -> JsResult<JsValue> {
        let message = self.0.next_message();
        serde_wasm_bindgen::to_value(&message).map_err(|e| serialization_error(e.to_string()))
    }

    #[wasm_bindgen(js_class = Query, js_name = source)]
    pub fn wasm_source(&self) -> String {
        self.0.source_info()
    }

    #[wasm_bindgen(js_class = Query, js_name = bind)]
    pub fn wasm_bind(&mut self, name: &str, term: JsValue) -> JsResult<()> {
        let term = serde_wasm_bindgen::from_value(term)?;
        self.0
            .bind(Symbol::new(name), term)
            .map_err(Error::from)
            .map_err(Error::into)
    }

    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen(js_class = Query, js_name = setLoggingOptions)]
    pub fn wasm_set_logging_options(
        &mut self,
        rust_log: Option<String>,
        polar_log: Option<String>,
    ) {
        self.0.set_logging_options(rust_log, polar_log);
    }
}
