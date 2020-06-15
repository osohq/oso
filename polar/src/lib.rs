#[macro_use]
pub mod macros;

#[cfg(test)]
#[macro_use]
extern crate maplit;

#[cfg(feature = "repl")]
pub mod cli;
mod debugger;
mod formatting;
mod lexer;
pub mod parser;
mod polar;
mod rewrites;
pub mod types;
mod vm;

pub use self::polar::{Polar, Query};
pub use formatting::{draw, ToPolarString};

use std::cell::RefCell;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::panic::catch_unwind;

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
        if let Ok(res) = catch_unwind(|| $body) {
            res
        } else {
            set_error(types::OperationalError::Unknown.into());
            // return as an int or a pointer
            POLAR_FAILURE as _
        }
    };
}

thread_local! {
    static LAST_ERROR: RefCell<Option<Box<types::PolarError>>> = RefCell::new(None);
}

fn set_error(e: types::PolarError) {
    LAST_ERROR.with(|prev| *prev.borrow_mut() = Some(Box::new(e)))
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
pub extern "C" fn polar_load(polar_ptr: *mut Polar, src: *const c_char) -> i32 {
    ffi_try!({
        let polar = unsafe { ffi_ref!(polar_ptr) };
        let src = unsafe { ffi_string!(src) };
        match polar.load_str(&src) {
            Err(err) => {
                set_error(err);
                POLAR_FAILURE
            }
            Ok(_) => POLAR_SUCCESS,
        }
    })
}

#[no_mangle]
pub extern "C" fn polar_check_inline_queries(polar_ptr: *mut Polar, query: *mut *mut Query) -> i32 {
    let mut query = if let Some(not_null) = std::ptr::NonNull::new(query) {
        not_null
    } else {
        set_error(
            types::ParameterError(String::from("Query out parameter cannot be null.")).into(),
        );
        return POLAR_FAILURE;
    };

    let result = catch_unwind(|| {
        let polar = unsafe { ffi_ref!(polar_ptr) };
        match polar.check_inline_queries() {
            Err(err) => {
                set_error(err);
                (null_mut(), POLAR_FAILURE)
            }
            Ok(Some(query)) => (box_ptr!(query), POLAR_SUCCESS),
            Ok(None) => (null_mut(), POLAR_SUCCESS),
        }
    });

    if let Ok((ret_query, ret_code)) = result {
        unsafe { *query.as_mut() = ret_query };
        ret_code
    } else {
        set_error(types::OperationalError::Unknown.into());
        unsafe { *query.as_mut() = null_mut() };
        POLAR_FAILURE
    }
}

#[no_mangle]
pub extern "C" fn polar_query_from_repl(polar_ptr: *mut Polar) -> *mut Query {
    ffi_try!({
        let polar = unsafe { ffi_ref!(polar_ptr) };
        match polar.new_query_from_repl() {
            Ok(query) => box_ptr!(query),
            Err(e) => {
                set_error(types::RuntimeError::Serialization { msg: e.to_string() }.into());
                null_mut()
            }
        }
    })
}

#[no_mangle]
/// Bools aren't portable, 0 means error 1 means success.
pub extern "C" fn polar_load_str(polar_ptr: *mut Polar, src: *const c_char) -> i32 {
    ffi_try!({
        let polar = unsafe { ffi_ref!(polar_ptr) };
        let s = unsafe { ffi_string!(src) };
        if let Err(e) = polar.load_str(&s) {
            set_error(e);
            POLAR_FAILURE
        } else {
            POLAR_SUCCESS
        }
    })
}

#[no_mangle]
pub extern "C" fn polar_new_query_from_term(
    polar_ptr: *mut Polar,
    query_term: *const c_char,
) -> *mut Query {
    ffi_try!({
        let polar = unsafe { ffi_ref!(polar_ptr) };
        let s = unsafe { ffi_string!(query_term) };
        let term = serde_json::from_str(&s);
        match term {
            Ok(term) => box_ptr!(polar.new_query_from_term(term)),
            Err(e) => {
                set_error(types::RuntimeError::Serialization { msg: e.to_string() }.into());
                null_mut()
            }
        }
    })
}

#[no_mangle]
pub extern "C" fn polar_new_query(polar_ptr: *mut Polar, query_str: *const c_char) -> *mut Query {
    ffi_try!({
        let polar = unsafe { ffi_ref!(polar_ptr) };
        let s = unsafe { ffi_string!(query_str) };
        let q = polar.new_query(&s);
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
pub extern "C" fn polar_query(polar_ptr: *mut Polar, query_ptr: *mut Query) -> *const c_char {
    ffi_try!({
        let polar = unsafe { ffi_ref!(polar_ptr) };
        let query = unsafe { ffi_ref!(query_ptr) };
        let event = polar.query(query);
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
/// - Polar.debug_command returns an error.
/// - Anything panics during the parsing/execution of the provided command.
#[no_mangle]
pub extern "C" fn polar_debug_command(
    polar_ptr: *mut Polar,
    query_ptr: *mut Query,
    value: *const c_char,
) -> i32 {
    ffi_try!({
        let polar = unsafe { ffi_ref!(polar_ptr) };
        let query = unsafe { ffi_ref!(query_ptr) };
        if !value.is_null() {
            let s = unsafe { ffi_string!(value) };
            let t = serde_json::from_str(&s);
            match t {
                Ok(types::Term {
                    value: types::Value::String(command),
                    ..
                }) => match polar.debug_command(query, command) {
                    Ok(_) => POLAR_SUCCESS,
                    Err(e) => {
                        set_error(e);
                        POLAR_FAILURE
                    }
                },
                Ok(_) => {
                    set_error(
                        types::RuntimeError::Serialization {
                            msg: "received bad command".to_string(),
                        }
                        .into(),
                    );
                    POLAR_FAILURE
                }
                Err(e) => {
                    set_error(types::RuntimeError::Serialization { msg: e.to_string() }.into());
                    POLAR_FAILURE
                }
            }
        } else {
            POLAR_FAILURE
        }
    })
}

#[no_mangle]
pub extern "C" fn polar_external_call_result(
    polar_ptr: *mut Polar,
    query_ptr: *mut Query,
    call_id: u64,
    value: *const c_char,
) -> i32 {
    ffi_try!({
        let polar = unsafe { ffi_ref!(polar_ptr) };
        let query = unsafe { ffi_ref!(query_ptr) };
        let mut term = None;
        if !value.is_null() {
            let s = unsafe { ffi_string!(value) };
            let t = serde_json::from_str(&s);
            match t {
                Ok(t) => term = Some(t),
                Err(e) => {
                    set_error(types::RuntimeError::Serialization { msg: e.to_string() }.into());
                    return POLAR_FAILURE;
                }
            }
        }
        match polar.external_call_result(query, call_id, term) {
            Ok(_) => POLAR_SUCCESS,
            Err(e) => {
                set_error(e);
                POLAR_FAILURE
            }
        }
    })
}

#[no_mangle]
pub extern "C" fn polar_external_question_result(
    polar_ptr: *mut Polar,
    query_ptr: *mut Query,
    call_id: u64,
    result: i32,
) -> i32 {
    ffi_try!({
        let polar = unsafe { ffi_ref!(polar_ptr) };
        let query = unsafe { ffi_ref!(query_ptr) };
        let result = result != POLAR_FAILURE;
        polar.external_question_result(query, call_id, result);
        POLAR_SUCCESS
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
