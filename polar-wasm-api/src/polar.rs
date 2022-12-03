use polar_core::{polar, sources::Source, terms::Symbol};
use wasm_bindgen::prelude::*;

use crate::errors::{serialization_error, Error};
use crate::JsResult;
use crate::Query;

#[wasm_bindgen]
pub struct Polar(polar::Polar);

#[allow(unused_variables)]
#[wasm_bindgen]
impl Polar {
    #[wasm_bindgen(constructor)]
    pub fn wasm_new() -> Self {
        console_error_panic_hook::set_once();
        Self(polar::Polar::new())
    }

    #[wasm_bindgen(js_class = Polar, js_name = load)]
    pub fn wasm_load(&self, sources: JsValue) -> JsResult<()> {
        let sources: Vec<Source> = serde_wasm_bindgen::from_value(sources)?;
        self.0
            .load(sources)
            .map_err(Error::from)
            .map_err(Error::into)
    }

    #[wasm_bindgen(js_class = Polar, js_name = clearRules)]
    pub fn wasm_clear_rules(&self) {
        self.0.clear_rules()
    }

    #[wasm_bindgen(js_class = Polar, js_name = registerConstant)]
    pub fn wasm_register_constant(&mut self, name: &str, term: JsValue) -> JsResult<()> {
        let term = serde_wasm_bindgen::from_value(term)?;
        self.0
            .register_constant(Symbol::new(name), term)
            .map_err(Error::from)?;
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
    pub fn wasm_new_query_from_term(&self, term: JsValue) -> JsResult<Query> {
        let term = serde_wasm_bindgen::from_value(term)?;
        Ok(Query::from(self.0.new_query_from_term(term, false)))
    }

    #[wasm_bindgen(js_class = Polar, js_name = newId)]
    pub fn wasm_get_external_id(&self) -> f64 {
        self.0.get_external_id() as f64
    }

    #[wasm_bindgen(js_class = Polar, js_name = nextMessage)]
    pub fn wasm_next_message(&self) -> JsResult<JsValue> {
        let message = self.0.next_message();
        serde_wasm_bindgen::to_value(&message).map_err(|e| serialization_error(e.to_string()))
    }

    #[wasm_bindgen(js_class = Polar, js_name = registerMro)]
    pub fn wasm_register_mro(&self, name: &str, mro: JsValue) -> JsResult<()> {
        let mro = serde_wasm_bindgen::from_value(mro)?;
        self.0
            .register_mro(Symbol::new(name), mro)
            .map_err(Error::from)
            .map_err(Error::into)
    }

    #[wasm_bindgen(js_class = Polar, js_name = buildDataFilter)]
    pub fn wasm_build_data_filter(
        &self,
        types: JsValue,
        partial_results: JsValue,
        variable: &str,
        class_tag: &str,
    ) -> JsResult<JsValue> {
        let types = serde_wasm_bindgen::from_value(types)?;
        let partial_results = serde_wasm_bindgen::from_value(partial_results)?;
        self.0
            .build_data_filter(types, partial_results, variable, class_tag)
            .map_err(Error::from)
            .map_err(Error::into)
            .and_then(|plan| {
                serde_wasm_bindgen::to_value(&plan).map_err(|e| serialization_error(e.to_string()))
            })
    }

    // TODO(@gkaemmer): this is a hack and should not be used for similar cases.
    // Ideally, we'd have a single "configuration" entrypoint for both the Polar
    // and Query types.
    #[wasm_bindgen(js_class = Polar, js_name = setIgnoreNoAllowWarning)]
    pub fn wasm_set_ignore_no_allow_warning(&mut self, ignore_no_allow_warning: bool) {
        self.0.set_ignore_no_allow_warning(ignore_no_allow_warning);
    }
}
