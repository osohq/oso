mod database;
mod diagnostics;
mod inspect;

use database::SourceMap;
use polar_core::polar;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// Equivalent to polar_core::error::Error
/// that additionally includes the kind and context fields
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PolarError {
    pub message: String,
    pub kind: polar_core::error::ErrorKind,
    pub context: Option<polar_core::error::ErrorContext>,
}

impl From<polar_core::error::PolarError> for PolarError {
    fn from(other: polar_core::error::PolarError) -> Self {
        Self {
            message: other.to_string(),
            kind: other.kind,
            context: other.context,
        }
    }
}

/// Converts a Rust value into a [`JsValue`].
fn to_value<T: Serialize>(value: &T) -> JsValue {
    serde_wasm_bindgen::to_value(value).unwrap_or_else(|_| "serialization error".into())
}

/// Wrapper for the `polar_core::Polar` type.
/// Used as the API interface for all the analytics
#[wasm_bindgen]
pub struct Polar {
    inner: polar::Polar,
    source_map: SourceMap,
}

#[wasm_bindgen]
impl Polar {
    #[wasm_bindgen(constructor)]
    pub fn wasm_new() -> Self {
        console_error_panic_hook::set_once();
        let inner = polar::Polar::new();
        let _ = inner.enable_roles();
        Self {
            inner,
            source_map: Default::default(),
        }
    }

    /// Loads a file into the knowledge base.
    ///
    /// In comparison to the `Polar` in the core, this
    /// will first remove the file.
    #[wasm_bindgen(js_class = Polar, js_name = load)]
    pub fn load(&self, src: &str, filename: &str) -> Result<(), JsValue> {
        let old = self.inner.remove_file(filename);
        self.inner
            .load(src, Some(filename.to_string()))
            .map_err(|e| {
                if let Some(old_src) = old {
                    self.inner
                        .load(&old_src, Some(filename.to_string()))
                        .expect("failed to reload old policy after new policy loading failed");
                }
                to_value(&PolarError::from(e))
            })?;
        let kb = self.inner.kb.read().unwrap();
        self.source_map.refresh(&kb, vec![filename]);
        Ok(())
    }

    #[wasm_bindgen(js_class = Polar, js_name = rename)]
    pub fn rename(&self, old_filename: &str, new_filename: &str) -> Result<(), JsValue> {
        if let Some(old) = self.inner.remove_file(old_filename) {
            self.source_map.remove_file(old_filename);
            self.load(&old, new_filename)
        } else {
            Ok(())
        }
    }

    #[wasm_bindgen(js_class = Polar, js_name = delete)]
    pub fn delete(&self, filename: &str) {
        self.source_map.remove_file(filename);
        let _old = self.inner.remove_file(filename);
    }

    #[wasm_bindgen(js_class = Polar, js_name = clearRules)]
    pub fn clear_rules(&self) {
        self.inner.clear_rules()
    }

    #[wasm_bindgen(js_class = Polar, js_name = getRuleInfo)]
    pub fn get_rule_info(&self, filename: &str) -> JsValue {
        to_value(&self.source_map.get_rule_info(filename))
    }

    #[wasm_bindgen(js_class = Polar, js_name = getTermInfo)]
    pub fn get_term_info(&self, filename: &str) -> JsValue {
        to_value(&self.source_map.get_term_info(filename))
    }

    #[wasm_bindgen(js_class = Polar, js_name = getParseErrors)]
    pub fn get_parse_errors(&self, src: &str) -> JsValue {
        to_value(&diagnostics::find_parse_errors(&src))
    }

    #[wasm_bindgen(js_class = Polar, js_name = getUnusedRules)]
    pub fn get_unused_rules(&self, src: &str) -> JsValue {
        let kb = self.inner.kb.read().unwrap();
        to_value(&diagnostics::find_unused_rules(&kb, src))
    }

    #[wasm_bindgen(js_class = Polar, js_name = getSymbolAt)]
    pub fn get_symbol_at(&self, filename: &str, location: usize) -> JsValue {
        to_value(&self.source_map.get_symbol_at(filename, location))
    }
}
