use polar_core::{polar, types::Symbol};
use wasm_bindgen::prelude::*;

use crate::errors::{serde_serialization_error, Error};
use crate::JsResult;
use crate::Query;

#[wasm_bindgen]
pub struct Polar(polar::Polar);

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
            Ok(term) => self.0.register_constant(Symbol::new(name), term),
            Err(e) => return Err(serde_serialization_error(e)),
        }
        Ok(())
    }

    #[wasm_bindgen(js_class = Polar, js_name = nextInlineQuery)]
    pub fn wasm_next_inline_query(&self) -> Option<Query> {
        self.0.next_inline_query(false).map(Query::from)
    }

    #[wasm_bindgen(js_class = Polar, js_name = newQueryFromStr)]
    pub fn wasm_new_query_from_str(&self, src: &str) -> JsResult<Query> {
        self.0
            .new_query(src, false)
            .map(Query::from)
            .map_err(Error::from)
            .map_err(Error::into)
    }

    #[wasm_bindgen(js_class = Polar, js_name = newQueryFromTerm)]
    pub fn wasm_new_query_from_term(&self, value: &str) -> JsResult<Query> {
        serde_json::from_str(value)
            .map(|term| Query::from(self.0.new_query_from_term(term, false)))
            .map_err(serde_serialization_error)
    }

    #[wasm_bindgen(js_class = Polar, js_name = newId)]
    pub fn wasm_get_external_id(&self) -> u64 {
        self.0.get_external_id()
    }
}
