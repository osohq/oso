// everything in this file is unsafe, clippy
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use polar_core::error::PolarError;
pub use polar_core::polar::Polar;
pub use polar_core::query::Query;
use polar_core::{error, terms};

use std::ffi::{c_void, CStr, CString};
use std::os::raw::c_char;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr::{null, null_mut};

/// Wrapper struct to help us return errors
#[repr(C)]
pub struct CResult<T> {
    pub result: *mut T,
    pub error: *const c_char,
}

impl From<Result<(), error::PolarError>> for CResult<c_void> {
    fn from(other: Result<(), error::PolarError>) -> Self {
        // Convenience handler to map `()` results to c_void
        Self::from(other.map(|_| null_mut()))
    }
}

impl<T> From<Result<*mut T, error::PolarError>> for CResult<T> {
    fn from(other: Result<*mut T, error::PolarError>) -> Self {
        match other {
            Ok(t) => Self {
                result: t,
                error: null(),
            },
            Err(e) => Self {
                result: null_mut(),
                error: {
                    let error_json = serde_json::to_string(&e).unwrap();
                    CString::new(error_json)
                        .expect("JSON should not contain any 0 bytes")
                        .into_raw()
                },
            },
        }
    }
}

/// Get a reference to an object from a pointer
macro_rules! ffi_ref {
    ($name:ident) => {{
        assert!(!$name.is_null());
        &mut *$name
    }};
}

/// Get a `Cow<str>` back from a C-style string
macro_rules! ffi_string {
    ($name:ident) => {{
        assert!(!$name.is_null());
        CStr::from_ptr($name).to_string_lossy()
    }};
}

/// Returns a raw pointer from an object
macro_rules! box_ptr {
    ($x:expr) => {
        Box::into_raw(Box::new($x))
    };
}

/// We use the convention of zero as an error term,
/// since we also use `null_ptr()` to indicate an error.
/// So for consistency, a zero term is an error in both cases.
pub const POLAR_FAILURE: i32 = 0;
pub const POLAR_SUCCESS: i32 = 1;

/// Unwrap the result term and return a zero/null pointer in the failure case
macro_rules! ffi_try {
    ($body:block) => {
        // $body.into()
        box_ptr!(catch_unwind(AssertUnwindSafe(|| $body))
            .map_err(|_| error::OperationalError::Unknown.into())
            .and_then(|res| res)
            .into())
    };
}

fn serde_error(e: serde_json::Error) -> PolarError {
    error::OperationalError::Serialization { msg: e.to_string() }.into()
}

fn from_json<T: serde::de::DeserializeOwned>(str: *const c_char) -> Result<T, PolarError> {
    let str = unsafe { ffi_string!(str) };
    serde_json::from_str(&str).map_err(serde_error)
}

#[no_mangle]
pub extern "C" fn polar_new() -> *mut Polar {
    box_ptr!(Polar::new())
}

#[no_mangle]
pub extern "C" fn polar_load(
    polar_ptr: *mut Polar,
    sources: *const c_char,
) -> *mut CResult<c_void> {
    ffi_try!({
        let polar = unsafe { ffi_ref!(polar_ptr) };
        from_json(sources).and_then(|sources| polar.load(sources))
    })
}

#[no_mangle]
pub extern "C" fn polar_clear_rules(polar_ptr: *mut Polar) -> *mut CResult<c_void> {
    ffi_try!({
        let polar = unsafe { ffi_ref!(polar_ptr) };
        polar.clear_rules();
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn polar_register_constant(
    polar_ptr: *mut Polar,
    name: *const c_char,
    value: *const c_char,
) -> *mut CResult<c_void> {
    ffi_try!({
        let polar = unsafe { ffi_ref!(polar_ptr) };
        let name = unsafe { ffi_string!(name) };
        from_json(value)
            .and_then(|value| polar.register_constant(terms::Symbol::new(name.as_ref()), value))
    })
}

#[no_mangle]
pub extern "C" fn polar_register_mro(
    polar_ptr: *mut Polar,
    name: *const c_char,
    mro: *const c_char,
) -> *mut CResult<c_void> {
    ffi_try!({
        let polar = unsafe { ffi_ref!(polar_ptr) };
        let name = unsafe { ffi_string!(name) };
        from_json(mro).and_then(|mro| polar.register_mro(terms::Symbol::new(name.as_ref()), mro))
    })
}
// @Note(steve): trace is treated as a bool. 0 for false, anything else for true.
// If we get more than one flag on these ffi methods, consider renaming it flags and making it a bitflags field.
// Then we won't have to update the ffi to add new optional things like logging or tracing or whatever.
#[no_mangle]
pub extern "C" fn polar_next_inline_query(polar_ptr: *mut Polar, trace: u32) -> *mut Query {
    let polar = unsafe { ffi_ref!(polar_ptr) };
    let trace = trace != 0;
    match polar.next_inline_query(trace) {
        Some(query) => box_ptr!(query),
        None => null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn polar_new_query_from_term(
    polar_ptr: *mut Polar,
    query_term: *const c_char,
    trace: u32,
) -> *mut CResult<Query> {
    ffi_try!({
        let polar = unsafe { ffi_ref!(polar_ptr) };
        from_json(query_term).map(|query| box_ptr!(polar.new_query_from_term(query, trace != 0)))
    })
}

#[no_mangle]
pub extern "C" fn polar_new_query(
    polar_ptr: *mut Polar,
    query_str: *const c_char,
    trace: u32,
) -> *mut CResult<Query> {
    ffi_try!({
        let polar = unsafe { ffi_ref!(polar_ptr) };
        let s = unsafe { ffi_string!(query_str) };
        let trace = trace != 0;
        polar.new_query(&s, trace).map(|q| box_ptr!(q))
    })
}

#[no_mangle]
pub extern "C" fn polar_next_polar_message(polar_ptr: *mut Polar) -> *mut CResult<c_char> {
    ffi_try!({
        let polar = unsafe { ffi_ref!(polar_ptr) };
        if let Some(msg) = polar.next_message() {
            let msg_json = serde_json::to_string(&msg).unwrap();
            Ok(CString::new(msg_json)
                .expect("JSON should not contain any 0 bytes")
                .into_raw())
        } else {
            Ok(null_mut())
        }
    })
}

#[no_mangle]
pub extern "C" fn polar_next_query_event(query_ptr: *mut Query) -> *mut CResult<c_char> {
    ffi_try!({
        let query = unsafe { ffi_ref!(query_ptr) };
        query.next_event().map(|event| {
            let event_json = serde_json::to_string(&event).unwrap();
            CString::new(event_json)
                .expect("JSON should not contain any 0 bytes")
                .into_raw()
        })
    })
}

/// Execute one debugger command for the given query.
///
/// ## Returns
/// - `0` on error.
/// - `1` on success.
///
/// ## Errors
/// - Provided value is NULL.
/// - Provided value contains malformed JSON.
/// - Provided value cannot be parsed to a Term wrapping a Value::String.
/// - Query.debug_command returns an error.
/// - Anything panics during the parsing/execution of the provided command.
#[no_mangle]
pub extern "C" fn polar_debug_command(
    query_ptr: *mut Query,
    value: *const c_char,
) -> *mut CResult<c_void> {
    ffi_try!({
        let query = unsafe { ffi_ref!(query_ptr) };
        from_json(value).and_then(|term: terms::Term| match term.value() {
            terms::Value::String(command) => query.debug_command(command),
            _ => Err(error::OperationalError::Serialization {
                msg: "received bad command".to_string(),
            }
            .into()),
        })
    })
}

#[no_mangle]
pub extern "C" fn polar_call_result(
    query_ptr: *mut Query,
    call_id: u64,
    term: *const c_char,
) -> *mut CResult<c_void> {
    ffi_try!({
        let query = unsafe { ffi_ref!(query_ptr) };
        from_json(term).and_then(|term| query.call_result(call_id, term))
    })
}

#[no_mangle]
pub extern "C" fn polar_question_result(
    query_ptr: *mut Query,
    call_id: u64,
    result: i32,
) -> *mut CResult<c_void> {
    ffi_try!({
        let query = unsafe { ffi_ref!(query_ptr) };
        let result = result != 0;
        query.question_result(call_id, result)
    })
}

#[no_mangle]
pub extern "C" fn polar_application_error(
    query_ptr: *mut Query,
    message: *mut c_char,
) -> *mut CResult<c_void> {
    ffi_try!({
        let query = unsafe { ffi_ref!(query_ptr) };
        let s = unsafe { ffi_string!(message) }.to_string();

        query.application_error(s)
    })
}

#[no_mangle]
pub extern "C" fn polar_next_query_message(query_ptr: *mut Query) -> *mut CResult<c_char> {
    ffi_try!({
        let query = unsafe { ffi_ref!(query_ptr) };
        if let Some(msg) = query.next_message() {
            let msg_json = serde_json::to_string(&msg).unwrap();
            Ok(CString::new(msg_json)
                .expect("JSON should not contain any 0 bytes")
                .into_raw())
        } else {
            Ok(null_mut())
        }
    })
}

#[no_mangle]
pub extern "C" fn polar_query_source_info(query_ptr: *mut Query) -> *mut CResult<c_char> {
    ffi_try!({
        let query = unsafe { ffi_ref!(query_ptr) };
        Ok(CString::new(query.source_info())
            .expect("No null bytes")
            .into_raw())
    })
}

#[no_mangle]
pub extern "C" fn polar_bind(
    query_ptr: *mut Query,
    name: *const c_char,
    value: *const c_char,
) -> *mut CResult<c_void> {
    ffi_try!({
        let query = unsafe { ffi_ref!(query_ptr) };
        let name = unsafe { ffi_string!(name) };
        from_json(value).and_then(|value| query.bind(terms::Symbol::new(name.as_ref()), value))
    })
}

#[no_mangle]
pub extern "C" fn polar_get_external_id(polar_ptr: *mut Polar) -> u64 {
    let polar = unsafe { ffi_ref!(polar_ptr) };
    polar.get_external_id()
}

/// Required to free strings properly
#[no_mangle]
pub extern "C" fn string_free(s: *mut c_char) -> i32 {
    if s.is_null() {
        return POLAR_FAILURE;
    }
    unsafe { CString::from_raw(s) };
    POLAR_SUCCESS
}

/// Recovers the original boxed version of `polar` so that
/// it can be properly freed
#[no_mangle]
pub extern "C" fn polar_free(polar: *mut Polar) -> i32 {
    std::mem::drop(unsafe { Box::from_raw(polar) });
    POLAR_SUCCESS
}

/// Recovers the original boxed version of `query` so that
/// it can be properly freed
#[no_mangle]
pub extern "C" fn query_free(query: *mut Query) -> i32 {
    std::mem::drop(unsafe { Box::from_raw(query) });
    POLAR_SUCCESS
}

/// Recovers the original boxed version of `result` so that
/// it can be properly freed
#[no_mangle]
pub extern "C" fn result_free(result: *mut CResult<c_void>) -> i32 {
    std::mem::drop(unsafe { Box::from_raw(result) });
    POLAR_SUCCESS
}

#[no_mangle]
pub extern "C" fn polar_build_data_filter(
    polar_ptr: *mut Polar,
    types: *const c_char,
    results: *const c_char,
    variable: *const c_char,
    class_tag: *const c_char,
) -> *mut CResult<c_char> {
    ffi_try!({
        let polar = unsafe { ffi_ref!(polar_ptr) };
        let variable = unsafe { ffi_string!(variable) };
        let class_tag = unsafe { ffi_string!(class_tag) };

        from_json(types)
            .and_then(|types| from_json(results).map(|results| (types, results)))
            .and_then(|(types, results)| {
                polar
                    .build_data_filter(types, results, &variable, &class_tag)
                    .map(|filter_plan| {
                        let plan_json = serde_json::to_string(&filter_plan).unwrap();
                        CString::new(plan_json)
                            .expect("JSON should not contain any 0 bytes")
                            .into_raw()
                    })
            })
    })
}

#[no_mangle]
pub extern "C" fn polar_build_filter_plan(
    polar_ptr: *mut Polar,
    types: *const c_char,
    results: *const c_char,
    variable: *const c_char,
    class_tag: *const c_char,
) -> *mut CResult<c_char> {
    ffi_try!({
        let polar = unsafe { ffi_ref!(polar_ptr) };
        let variable = unsafe { ffi_string!(variable) };
        let class_tag = unsafe { ffi_string!(class_tag) };

        from_json(types)
            .and_then(|types| from_json(results).map(|results| (types, results)))
            .and_then(|(types, results)| {
                polar
                    .build_filter_plan(types, results, &variable, &class_tag)
                    .map(|filter_plan| {
                        let plan_json = serde_json::to_string(&filter_plan).unwrap();
                        CString::new(plan_json)
                            .expect("JSON should not contain any 0 bytes")
                            .into_raw()
                    })
            })
    })
}
