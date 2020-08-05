mod errors;
mod polar;
mod query;

pub use polar::Polar;
pub use query::Query;

// TODO(gj): figure out how to handle Rust panics in wasm.

type JsResult<T> = Result<T, wasm_bindgen::JsValue>;
