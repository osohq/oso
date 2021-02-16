use wasm_bindgen::JsValue;

use polar_core::error::{
    ErrorKind, FormattedPolarError, OperationalError, ParameterError, ParseError, PolarError,
    RuntimeError,
};

pub struct Error {
    pub kind: String,
    inner: FormattedPolarError,
}

pub fn serde_serialization_error(e: serde_json::Error) -> JsValue {
    serialization_error(e.to_string())
}

pub fn serialization_error(msg: String) -> JsValue {
    Error::from(PolarError::from(RuntimeError::Serialization { msg })).into()
}

fn kind(err: &PolarError) -> String {
    use ErrorKind::*;
    use OperationalError::*;
    use ParseError::*;
    use RuntimeError::*;
    match err.kind {
        Parse(IntegerOverflow { .. }) => "ParseError::IntegerOverflow",
        Parse(InvalidTokenCharacter { .. }) => "ParseError::InvalidTokenCharacter",
        Parse(InvalidToken { .. }) => "ParseError::InvalidToken",
        Parse(UnrecognizedEOF { .. }) => "ParseError::UnrecognizedEOF",
        Parse(UnrecognizedToken { .. }) => "ParseError::UnrecognizedToken",
        Parse(ExtraToken { .. }) => "ParseError::ExtraToken",
        Parse(ReservedWord { .. }) => "ParseError::ReservedWord",
        Parse(InvalidFloat { .. }) => "ParseError::InvalidFloat",
        Parse(WrongValueType { .. }) => "ParseError::WrongValueType",
        Runtime(Application { .. }) => "RuntimeError::Application",
        Runtime(ArithmeticError { .. }) => "RuntimeError::ArithmeticError",
        Runtime(FileLoading { .. }) => "RuntimeError::FileLoading",
        Runtime(IncompatibleBindings { .. }) => "RuntimeError::IncompatibleBindings",
        Runtime(QueryTimeout { .. }) => "RuntimeError::QueryTimeout",
        Runtime(Serialization { .. }) => "RuntimeError::Serialization",
        Runtime(StackOverflow { .. }) => "RuntimeError::StackOverflow",
        Runtime(TypeError { .. }) => "RuntimeError::TypeError",
        Runtime(UnboundVariable { .. }) => "RuntimeError::UnboundVariable",
        Runtime(Unsupported { .. }) => "RuntimeError::Unsupported",
        Operational(Unimplemented(..)) => "OperationalError::Unimplemented",
        Operational(Unknown) => "OperationalError::Unknown",
        Operational(InvalidState(..)) => "OperationalError::InvalidState",
        Parameter(ParameterError(..)) => "ParameterError::ParameterError",
    }
    .to_owned()
}

impl From<PolarError> for Error {
    fn from(err: PolarError) -> Self {
        let kind = kind(&err);
        Self {
            inner: err.into(),
            kind,
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
