pub use polar_core::polar::{Polar, Query};
use polar_core::{error, terms};

use std::cell::RefCell;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr::{null, null_mut};

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
        if let Ok(res) = catch_unwind(AssertUnwindSafe(|| $body)) {
            res
        } else {
            // return as an int or a pointer
            set_error(error::OperationalError::Unknown.into()) as _
        }
    };
}

thread_local! {
    static LAST_ERROR: RefCell<Option<Box<error::PolarError>>> = RefCell::new(None);
}

fn set_error(e: error::PolarError) -> i32 {
    LAST_ERROR.with(|prev| *prev.borrow_mut() = Some(Box::new(e)));
    POLAR_FAILURE
}

#[no_mangle]
pub extern "C" fn polar_get_error() -> *const c_char {
    ffi_try!({
        let err = LAST_ERROR.with(|prev| prev.borrow_mut().take());
        if let Some(e) = err {
            let error_json = serde_json::to_string(&e).unwrap();
            CString::new(error_json)
                .expect("JSON should not contain any 0 bytes")
                .into_raw()
        } else {
            null()
        }
    })
}

#[no_mangle]
pub extern "C" fn polar_new() -> *mut Polar {
    ffi_try!({ box_ptr!(Polar::new()) })
}

#[no_mangle]
pub extern "C" fn polar_load(polar_ptr: *mut Polar, sources: *const c_char) -> i32 {
    ffi_try!({
        let polar = unsafe { ffi_ref!(polar_ptr) };
        let sources = unsafe { ffi_string!(sources) };
        let sources = serde_json::from_str(&sources);
        match sources {
            Ok(sources) => match polar.load(sources) {
                Err(err) => set_error(err),
                Ok(_) => POLAR_SUCCESS,
            },
            Err(e) => set_error(error::RuntimeError::Serialization { msg: e.to_string() }.into()),
        }
    })
}

#[no_mangle]
pub extern "C" fn polar_clear_rules(polar_ptr: *mut Polar) -> i32 {
    ffi_try!({
        let polar = unsafe { ffi_ref!(polar_ptr) };
        polar.clear_rules();
        POLAR_SUCCESS
    })
}

#[no_mangle]
pub extern "C" fn polar_register_constant(
    polar_ptr: *mut Polar,
    name: *const c_char,
    value: *const c_char,
) -> i32 {
    ffi_try!({
        let polar = unsafe { ffi_ref!(polar_ptr) };
        let name = unsafe { ffi_string!(name) };
        let value = unsafe { ffi_string!(value) };
        let value = serde_json::from_str(&value);
        match value {
            Ok(value) => match polar.register_constant(terms::Symbol::new(name.as_ref()), value) {
                Err(e) => {
                    set_error(e);
                    POLAR_FAILURE
                }
                Ok(()) => POLAR_SUCCESS,
            },
            Err(e) => set_error(error::RuntimeError::Serialization { msg: e.to_string() }.into()),
        }
    })
}

#[no_mangle]
pub extern "C" fn polar_register_mro(
    polar_ptr: *mut Polar,
    name: *const c_char,
    mro: *const c_char,
) -> i32 {
    ffi_try!({
        let polar = unsafe { ffi_ref!(polar_ptr) };
        let name = unsafe { ffi_string!(name) };
        let mro = unsafe { ffi_string!(mro) };
        let mro = serde_json::from_str(&mro);
        match mro {
            Ok(mro) => match polar.register_mro(terms::Symbol::new(name.as_ref()), mro) {
                Err(e) => {
                    set_error(e);
                    POLAR_FAILURE
                }
                Ok(()) => POLAR_SUCCESS,
            },
            Err(e) => set_error(error::RuntimeError::Serialization { msg: e.to_string() }.into()),
        }
    })
}
// @Note(steve): trace is treated as a bool. 0 for false, anything else for true.
// If we get more than one flag on these ffi methods, consider renaming it flags and making it a bitflags field.
// Then we wont have to update the ffi to add new optional things like logging or tracing or whatever.
#[no_mangle]
pub extern "C" fn polar_next_inline_query(polar_ptr: *mut Polar, trace: u32) -> *mut Query {
    ffi_try!({
        let polar = unsafe { ffi_ref!(polar_ptr) };
        let trace = trace != 0;
        match polar.next_inline_query(trace) {
            Some(query) => box_ptr!(query),
            None => null_mut(),
        }
    })
}

#[no_mangle]
pub extern "C" fn polar_new_query_from_term(
    polar_ptr: *mut Polar,
    query_term: *const c_char,
    trace: u32,
) -> *mut Query {
    ffi_try!({
        let polar = unsafe { ffi_ref!(polar_ptr) };
        let s = unsafe { ffi_string!(query_term) };
        let term = serde_json::from_str(&s);
        let trace = trace != 0;
        match term {
            Ok(term) => box_ptr!(polar.new_query_from_term(term, trace)),
            Err(e) => {
                set_error(error::RuntimeError::Serialization { msg: e.to_string() }.into());
                null_mut()
            }
        }
    })
}

#[no_mangle]
pub extern "C" fn polar_new_query(
    polar_ptr: *mut Polar,
    query_str: *const c_char,
    trace: u32,
) -> *mut Query {
    ffi_try!({
        let polar = unsafe { ffi_ref!(polar_ptr) };
        let s = unsafe { ffi_string!(query_str) };
        let trace = trace != 0;
        let q = polar.new_query(&s, trace);
        match q {
            Ok(q) => box_ptr!(q),
            Err(e) => {
                set_error(e);
                null_mut()
            }
        }
    })
}

#[no_mangle]
pub extern "C" fn polar_next_polar_message(polar_ptr: *mut Polar) -> *const c_char {
    ffi_try!({
        let polar = unsafe { ffi_ref!(polar_ptr) };
        if let Some(msg) = polar.next_message() {
            let msg_json = serde_json::to_string(&msg).unwrap();
            CString::new(msg_json)
                .expect("JSON should not contain any 0 bytes")
                .into_raw()
        } else {
            null()
        }
    })
}

#[no_mangle]
pub extern "C" fn polar_next_query_event(query_ptr: *mut Query) -> *const c_char {
    ffi_try!({
        let query = unsafe { ffi_ref!(query_ptr) };
        let event = query.next_event();
        match event {
            Ok(event) => {
                let event_json = serde_json::to_string(&event).unwrap();
                CString::new(event_json)
                    .expect("JSON should not contain any 0 bytes")
                    .into_raw()
            }
            Err(e) => {
                set_error(e);
                null()
            }
        }
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
pub extern "C" fn polar_debug_command(query_ptr: *mut Query, value: *const c_char) -> i32 {
    ffi_try!({
        let query = unsafe { ffi_ref!(query_ptr) };
        if !value.is_null() {
            let s = unsafe { ffi_string!(value) };
            let t = serde_json::from_str(&s);
            match t.as_ref().map(terms::Term::value) {
                Ok(terms::Value::String(command)) => match query.debug_command(command) {
                    Ok(_) => POLAR_SUCCESS,
                    Err(e) => set_error(e),
                },
                Ok(_) => set_error(
                    error::RuntimeError::Serialization {
                        msg: "received bad command".to_string(),
                    }
                    .into(),
                ),
                Err(e) => {
                    set_error(error::RuntimeError::Serialization { msg: e.to_string() }.into())
                }
            }
        } else {
            POLAR_FAILURE
        }
    })
}

#[no_mangle]
pub extern "C" fn polar_call_result(
    query_ptr: *mut Query,
    call_id: u64,
    value: *const c_char,
) -> i32 {
    ffi_try!({
        let query = unsafe { ffi_ref!(query_ptr) };
        let mut term = None;
        if !value.is_null() {
            let s = unsafe { ffi_string!(value) };
            let t = serde_json::from_str(&s);
            match t {
                Ok(t) => term = Some(t),
                Err(e) => {
                    return set_error(
                        error::RuntimeError::Serialization { msg: e.to_string() }.into(),
                    );
                }
            }
        }
        match query.call_result(call_id, term) {
            Ok(_) => POLAR_SUCCESS,
            Err(e) => set_error(e),
        }
    })
}

#[no_mangle]
pub extern "C" fn polar_question_result(query_ptr: *mut Query, call_id: u64, result: i32) -> i32 {
    ffi_try!({
        let query = unsafe { ffi_ref!(query_ptr) };
        let result = result != POLAR_FAILURE;
        match query.question_result(call_id, result) {
            Ok(_) => POLAR_SUCCESS,
            Err(e) => set_error(e),
        }
    })
}

#[no_mangle]
pub extern "C" fn polar_application_error(query_ptr: *mut Query, message: *mut c_char) -> i32 {
    ffi_try!({
        let query = unsafe { ffi_ref!(query_ptr) };
        let s = if !message.is_null() {
            unsafe { ffi_string!(message) }.to_string()
        } else {
            "".to_owned()
        };

        match query.application_error(s) {
            Ok(_) => POLAR_SUCCESS,
            Err(e) => set_error(e),
        }
    })
}

#[no_mangle]
pub extern "C" fn polar_next_query_message(query_ptr: *mut Query) -> *const c_char {
    ffi_try!({
        let query = unsafe { ffi_ref!(query_ptr) };
        if let Some(msg) = query.next_message() {
            let msg_json = serde_json::to_string(&msg).unwrap();
            CString::new(msg_json)
                .expect("JSON should not contain any 0 bytes")
                .into_raw()
        } else {
            null()
        }
    })
}

#[no_mangle]
pub extern "C" fn polar_query_source_info(query_ptr: *mut Query) -> *const c_char {
    ffi_try!({
        let query = unsafe { ffi_ref!(query_ptr) };
        CString::new(query.source_info())
            .expect("No null bytes")
            .into_raw()
    })
}

#[no_mangle]
pub extern "C" fn polar_bind(
    query_ptr: *mut Query,
    name: *const c_char,
    value: *const c_char,
) -> i32 {
    ffi_try!({
        let query = unsafe { ffi_ref!(query_ptr) };
        let name = unsafe { ffi_string!(name) };
        let value = unsafe { ffi_string!(value) };
        let value = serde_json::from_str(&value);
        match value {
            Ok(value) => match query.bind(terms::Symbol::new(name.as_ref()), value) {
                Ok(_) => POLAR_SUCCESS,
                Err(e) => set_error(e),
            },
            Err(e) => set_error(error::RuntimeError::Serialization { msg: e.to_string() }.into()),
        }
    })
}

#[no_mangle]
pub extern "C" fn polar_get_external_id(polar_ptr: *mut Polar) -> u64 {
    ffi_try!({
        let polar = unsafe { ffi_ref!(polar_ptr) };
        polar.get_external_id()
    })
}

/// Required to free strings properly
#[no_mangle]
pub extern "C" fn string_free(s: *mut c_char) -> i32 {
    ffi_try!({
        if s.is_null() {
            return POLAR_FAILURE;
        }
        unsafe { CString::from_raw(s) };
        POLAR_SUCCESS
    })
}

/// Recovers the original boxed version of `polar` so that
/// it can be properly freed
#[no_mangle]
pub extern "C" fn polar_free(polar: *mut Polar) -> i32 {
    ffi_try!({
        std::mem::drop(unsafe { Box::from_raw(polar) });
        POLAR_SUCCESS
    })
}

/// Recovers the original boxed version of `query` so that
/// it can be properly freed
#[no_mangle]
pub extern "C" fn query_free(query: *mut Query) -> i32 {
    ffi_try!({
        std::mem::drop(unsafe { Box::from_raw(query) });
        POLAR_SUCCESS
    })
}

#[no_mangle]
pub extern "C" fn polar_build_filter_plan(
    polar_ptr: *mut Polar,
    types: *const c_char,
    results: *const c_char,
    variable: *const c_char,
    class_tag: *const c_char,
) -> *const c_char {
    ffi_try!({
        let polar = unsafe { ffi_ref!(polar_ptr) };

        let types_str = unsafe { ffi_string!(types) };
        let results_str = unsafe { ffi_string!(results) };
        let types = match serde_json::from_str(&types_str)
            .map_err(|e| error::RuntimeError::Serialization { msg: e.to_string() }.into())
        {
            Ok(types) => types,
            Err(e) => {
                set_error(e);
                return null();
            }
        };
        let partial_results = match serde_json::from_str(&results_str)
            .map_err(|e| error::RuntimeError::Serialization { msg: e.to_string() }.into())
        {
            Ok(partial_results) => partial_results,
            Err(e) => {
                set_error(e);
                return null();
            }
        };

        let variable = unsafe { ffi_string!(variable) };
        let class_tag = unsafe { ffi_string!(class_tag) };

        let filter_plan = polar.build_filter_plan(types, partial_results, &variable, &class_tag);
        match filter_plan {
            Ok(filter_plan) => {
                let plan_json = serde_json::to_string(&filter_plan).unwrap();
                CString::new(plan_json)
                    .expect("JSON should not contain any 0 bytes")
                    .into_raw()
            }
            Err(e) => {
                set_error(e);
                null()
            }
        }
    })
}
