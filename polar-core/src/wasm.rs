use js_sys::Error;
use wasm_bindgen::prelude::*;

use super::error::{
    ErrorKind, OperationalError, ParameterError, ParseError, PolarError, RuntimeError,
};
use super::polar::{self, Query};
use super::types::Term;

// #[cfg(target_arch = "wasm32")]
impl From<PolarError> for Error {
    fn from(err: PolarError) -> Error {
        let e = Error::new(&err.formatted);
        e.set_name(err.kind.name());
        e
    }
}

impl ErrorKind {
    fn name(&self) -> &'static str {
        match self {
            Self::Parse(e) => e.name(),
            Self::Runtime(e) => e.name(),
            Self::Operational(e) => e.name(),
            Self::Parameter(e) => e.name(),
        }
    }
}

type JsResult<T> = Result<T, JsValue>;

impl ParseError {
    fn name(&self) -> &'static str {
        match self {
            Self::IntegerOverflow { .. } => "ParseError::IntegerOverflow",
            Self::InvalidTokenCharacter { .. } => "ParseError::InvalidTokenCharacter",
            Self::InvalidToken { .. } => "ParseError::InvalidToken",
            Self::UnrecognizedEOF { .. } => "ParseError::UnrecognizedEOF",
            Self::UnrecognizedToken { .. } => "ParseError::UnrecognizedToken",
            Self::ExtraToken { .. } => "ParseError::ExtraToken",
            Self::ReservedWord { .. } => "ParseError::ReservedWord",
            Self::InvalidFloat { .. } => "ParseError::InvalidFloat",
        }
    }
}

impl RuntimeError {
    fn name(&self) -> &'static str {
        match self {
            Self::Application { .. } => "RuntimeError::Application",
            Self::ArithmeticError { .. } => "RuntimeError::ArithmeticError",
            Self::QueryTimeout { .. } => "RuntimeError::QueryTimeout",
            Self::Serialization { .. } => "RuntimeError::Serialization",
            Self::StackOverflow { .. } => "RuntimeError::StackOverflow",
            Self::TypeError { .. } => "RuntimeError::TypeError",
            Self::UnboundVariable { .. } => "RuntimeError::UnboundVariable",
            Self::Unsupported { .. } => "RuntimeError::Unsupported",
        }
    }
}

impl OperationalError {
    fn name(&self) -> &'static str {
        match self {
            Self::Unimplemented { .. } => "OperationalError::Unimplemented",
            Self::Unknown => "OperationalError::Unknown",
        }
    }
}

impl ParameterError {
    const fn name(&self) -> &'static str {
        "ParameterError"
    }
}

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
            .map_err(|e| Error::from(e).into())
    }

    #[wasm_bindgen(js_class = Polar, js_name = nextInlineQuery)]
    pub fn wasm_next_inline_query(&self) -> Option<Query> {
        self.0.next_inline_query(false)
    }

    #[wasm_bindgen(js_class = Polar, js_name = newQueryFromStr)]
    pub fn wasm_new_query_from_str(&self, src: &str) -> JsResult<Query> {
        self.0
            .new_query(src, false)
            .map_err(|e| Error::from(e).into())
    }

    #[wasm_bindgen(js_class = Polar, js_name = newQueryFromTerm)]
    pub fn wasm_new_query_from_term(&self, term: Term) -> Query {
        self.0.new_query_from_term(term, false)
    }

    #[wasm_bindgen(js_class = Polar, js_name = newId)]
    pub fn wasm_get_external_id(&self) -> u64 {
        self.0.get_external_id()
    }
}

#[wasm_bindgen]
pub enum QueryEvent {
    None,
    Debug,
    Done,
    MakeExternal,
    ExternalCall,
    ExternalIsa,
    ExternalIsSubSpecializer,
    Result,
}

#[wasm_bindgen]
impl Query {
    #[wasm_bindgen(js_name = nextEvent)]
    pub fn wasm_next_event(&mut self) -> JsResult<JsValue> {
        self.next_event()
            .map_err(|e| Error::from(e).into())
            .and_then(|event| serde_wasm_bindgen::to_value(&event).map_err(|e| e.into()))
    }

    #[wasm_bindgen(js_name = callResult)]
    pub fn wasm_call_result(&mut self, call_id: u64, value: Option<Term>) -> JsResult<()> {
        self.call_result(call_id, value)
            .map_err(|e| Error::from(e).into())
    }

    #[wasm_bindgen(js_name = questionResult)]
    pub fn wasm_question_result(&mut self, call_id: u64, result: bool) {
        self.question_result(call_id, result)
    }

    #[wasm_bindgen(js_name = debugCommand)]
    pub fn wasm_debug_command(&mut self, command: &str) -> JsResult<()> {
        self.debug_command(command)
            .map_err(|e| Error::from(e).into())
    }
}
