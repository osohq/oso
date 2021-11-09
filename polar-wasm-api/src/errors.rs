use wasm_bindgen::JsValue;

use polar_core::error::{
    ErrorKind, FormattedPolarError, OperationalError, ParseError, PolarError, RuntimeError,
    ValidationError,
};

pub struct Error {
    pub kind: String,
    inner: FormattedPolarError,
}

pub fn serialization_error(msg: String) -> JsValue {
    Error::from(PolarError::from(OperationalError::Serialization { msg })).into()
}

fn kind(err: &PolarError) -> String {
    use ErrorKind::*;
    use OperationalError::*;
    use ParseError::*;
    use RuntimeError::*;
    use ValidationError::*;
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
        Parse(DuplicateKey { .. }) => "ParseError::DuplicateKey",
        Runtime(Application { .. }) => "RuntimeError::Application",
        Runtime(ArithmeticError { .. }) => "RuntimeError::ArithmeticError",
        Runtime(FileLoading { .. }) => "RuntimeError::FileLoading",
        Runtime(IncompatibleBindings { .. }) => "RuntimeError::IncompatibleBindings",
        Runtime(QueryTimeout { .. }) => "RuntimeError::QueryTimeout",
        Runtime(StackOverflow { .. }) => "RuntimeError::StackOverflow",
        Runtime(TypeError { .. }) => "RuntimeError::TypeError",
        Runtime(UnhandledPartial { .. }) => "RuntimeError::UnhandledPartial",
        Runtime(Unsupported { .. }) => "RuntimeError::Unsupported",
        Runtime(DataFilteringFieldMissing { .. }) => "RuntimeError::DataFilteringFieldMissing",
        Operational(Serialization { .. }) => "OperationalError::Serialization",
        Operational(Unimplemented { .. }) => "OperationalError::Unimplemented",
        Operational(Unknown) => "OperationalError::Unknown",
        Operational(InvalidState { .. }) => "OperationalError::InvalidState",
        Validation(InvalidRule { .. }) => "ValidationError::InvalidRule",
        Validation(InvalidRuleType { .. }) => "ValidationError::InvalidRuleType",
        Validation(ResourceBlock { .. }) => "ValidationError::ResourceBlock",
        Validation(UndefinedRuleCall { .. }) => "ValidationError::UndefinedRuleCall",
        Validation(SingletonVariable { .. }) => "ValidationError::SingletonVariable",
        Validation(UnregisteredClass { .. }) => "ValidationError::UnregisteredClass",
        Validation(MissingRequiredRule { .. }) => "ValidationError::MissingRequiredRule",
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
