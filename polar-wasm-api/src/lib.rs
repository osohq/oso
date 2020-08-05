use wasm_bindgen::prelude::*;

use polar_core::error::{
    ErrorKind, OperationalError, ParameterError, ParseError, PolarError, RuntimeError,
};
use polar_core::{polar, types};

// TODO(gj): figure out how to handle Rust panics in wasm.

pub struct Error {
    pub kind: String,
    inner: PolarError,
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
        Runtime(Application { .. }) => "RuntimeError::Application",
        Runtime(ArithmeticError { .. }) => "RuntimeError::ArithmeticError",
        Runtime(QueryTimeout { .. }) => "RuntimeError::QueryTimeout",
        Runtime(Serialization { .. }) => "RuntimeError::Serialization",
        Runtime(StackOverflow { .. }) => "RuntimeError::StackOverflow",
        Runtime(TypeError { .. }) => "RuntimeError::TypeError",
        Runtime(UnboundVariable { .. }) => "RuntimeError::UnboundVariable",
        Runtime(Unsupported { .. }) => "RuntimeError::Unsupported",
        Operational(Unimplemented(..)) => "OperationalError::Unimplemented",
        Operational(Unknown) => "OperationalError::Unknown",
        Parameter(ParameterError(..)) => "ParameterError::ParameterError",
    }
    .to_owned()
}

impl From<PolarError> for Error {
    fn from(err: PolarError) -> Self {
        let kind = kind(&err);
        Self { inner: err, kind }
    }
}

impl From<Error> for js_sys::Error {
    fn from(err: Error) -> Self {
        let e = Self::new(&err.inner.formatted);
        e.set_name(&err.kind);
        e
    }
}

impl From<Error> for wasm_bindgen::JsValue {
    fn from(err: Error) -> Self {
        js_sys::Error::from(err).into()
    }
}

type JsResult<T> = Result<T, JsValue>;

#[wasm_bindgen]
pub struct Polar(polar::Polar);

#[wasm_bindgen]
pub struct Query(polar::Query);

#[wasm_bindgen]
pub struct Term(types::Term);

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
            .map_err(Error::from)
            .map_err(Error::into)
    }

    #[wasm_bindgen(js_class = Polar, js_name = registerConstant)]
    pub fn wasm_register_constant(&mut self, name: &str, value: Term) {
        self.0.register_constant(types::Symbol::new(name), value.0)
    }

    #[wasm_bindgen(js_class = Polar, js_name = nextInlineQuery)]
    pub fn wasm_next_inline_query(&self) -> Option<Query> {
        self.0.next_inline_query(false).map(Query)
    }

    #[wasm_bindgen(js_class = Polar, js_name = newQueryFromStr)]
    pub fn wasm_new_query_from_str(&self, src: &str) -> JsResult<Query> {
        self.0
            .new_query(src, false)
            .map(Query)
            .map_err(Error::from)
            .map_err(Error::into)
    }

    #[wasm_bindgen(js_class = Polar, js_name = newQueryFromTerm)]
    pub fn wasm_new_query_from_term(&self, term: Term) -> Query {
        Query(self.0.new_query_from_term(term.0, false))
    }

    #[wasm_bindgen(js_class = Polar, js_name = newId)]
    pub fn wasm_get_external_id(&self) -> u64 {
        self.0.get_external_id()
    }
}

#[wasm_bindgen]
impl Query {
    #[wasm_bindgen(js_class = Query, js_name = nextEvent)]
    pub fn wasm_next_event(&mut self) -> JsResult<JsValue> {
        self.0
            .next_event()
            .map_err(Error::from)
            .map_err(Error::into)
            .and_then(|event| serde_wasm_bindgen::to_value(&event).map_err(|e| e.into()))
    }

    #[wasm_bindgen(js_class = Query, js_name = callResult)]
    pub fn wasm_call_result(&mut self, call_id: u64, value: Option<String>) -> JsResult<()> {
        let term = value.and_then(|v| serde_json::from_str(&v).ok());
        self.0
            .call_result(call_id, term)
            .map_err(Error::from)
            .map_err(Error::into)
    }

    #[wasm_bindgen(js_class = Query, js_name = questionResult)]
    pub fn wasm_question_result(&mut self, call_id: u64, result: bool) {
        self.0.question_result(call_id, result)
    }

    #[wasm_bindgen(js_class = Query, js_name = debugCommand)]
    pub fn wasm_debug_command(&mut self, command: &str) -> JsResult<()> {
        self.0
            .debug_command(command)
            .map_err(Error::from)
            .map_err(Error::into)
    }

    #[wasm_bindgen(js_class = Query, js_name = appError)]
    pub fn wasm_app_error(&mut self, msg: &str) {
        self.0.application_error(msg.to_owned())
    }
}
