mod errors;
mod polar;
mod query;

pub use polar::Polar;
pub use query::Query;

type JsResult<T> = Result<T, wasm_bindgen::JsValue>;
