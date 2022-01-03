use wasm_bindgen::JsValue;

use polar_core::error::{OperationalError, PolarError};

pub(crate) struct Error(PolarError);

pub(crate) fn serialization_error(msg: String) -> JsValue {
    Error(OperationalError::Serialization { msg }.into()).into()
}

impl From<PolarError> for Error {
    fn from(other: PolarError) -> Self {
        Self(other)
    }
}

impl From<Error> for js_sys::Error {
    fn from(err: Error) -> Self {
        let e = Self::new(&err.0.to_string());
        e.set_name(&err.0.kind());
        e
    }
}

impl From<Error> for JsValue {
    fn from(err: Error) -> Self {
        js_sys::Error::from(err).into()
    }
}
