use wasm_bindgen::JsValue;

use polar_core::error::{FormattedPolarError, OperationalError, PolarError};

pub struct Error {
    pub kind: String,
    inner: FormattedPolarError,
}

pub fn serialization_error(msg: String) -> JsValue {
    Error::from(PolarError::from(OperationalError::Serialization { msg })).into()
}

impl From<PolarError> for Error {
    fn from(err: PolarError) -> Self {
        Self {
            kind: err.kind(),
            inner: err.into(),
        }
    }
}

impl From<Error> for js_sys::Error {
    fn from(err: Error) -> Self {
        let e = Self::new(&err.inner.formatted);
        e.set_name(&err.kind);
        e
    }
}

impl From<Error> for JsValue {
    fn from(err: Error) -> Self {
        js_sys::Error::from(err).into()
    }
}
