use wasm_bindgen::JsValue;

use polar_core::error::{FormattedPolarError, OperationalError, PolarError};

pub(crate) struct Error(FormattedPolarError);

pub(crate) fn serialization_error(msg: String) -> JsValue {
    Error(PolarError::from(OperationalError::Serialization { msg }).into()).into()
}

impl From<PolarError> for Error {
    fn from(other: PolarError) -> Self {
        Self(other.into())
    }
}

impl From<Error> for js_sys::Error {
    fn from(err: Error) -> Self {
        let e = Self::new(&err.0.message);
        e.set_name(&err.0.kind);
        e
    }
}

impl From<Error> for JsValue {
    fn from(err: Error) -> Self {
        js_sys::Error::from(err).into()
    }
}
