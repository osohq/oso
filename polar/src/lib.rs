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

pub use self::polar::{Load, Polar, Query};
pub use formatting::ToPolarString;

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

/// We use a non-standard convention of zero as an error term,
/// since we also use `null_ptr()` to indicate an error.
/// So for consistency, a zero term is an error in both cases.
const EXIT_FAILURE: i32 = 0;
const EXIT_SUCCESS: i32 = 1;

/// Unwrap the result term and return a zero/null pointer in the failure case
macro_rules! ffi_try {
    ($body:block) => {
        if let Ok(res) = catch_unwind(|| $body) {
            res
        } else {
            set_error(types::OperationalError::Unknown.into());
            // return as an int or a pointer
            EXIT_FAILURE as _
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

/// Create a new Load struct from a load string.
///
/// Returns: A null ptr on error, otherwise a Load struct (must be freed by caller).
#[no_mangle]
pub extern "C" fn polar_new_load(polar_ptr: *mut Polar, src: *const c_char) -> *mut Load {
    ffi_try!({
        let polar = unsafe { ffi_ref!(polar_ptr) };
        let query_str = unsafe { ffi_string!(src) };
        match polar.new_load(&query_str) {
            Err(err) => {
                set_error(err);
                null_mut()
            }
            Ok(load) => box_ptr!(load),
        }
    })
}

#[no_mangle]
pub extern "C" fn polar_load(
    polar_ptr: *mut Polar,
    load: *mut Load,
    query: *mut *mut Query,
) -> i32 {
    let mut query = if let Some(not_null) = std::ptr::NonNull::new(query) {
        not_null
    } else {
        set_error(
            types::ParameterError(String::from("Query out parameter cannot be null.")).into(),
        );
        return EXIT_FAILURE;
    };

    let result = catch_unwind(|| {
        let polar = unsafe { ffi_ref!(polar_ptr) };
        let load = unsafe { ffi_ref!(load) };
        match polar.load(load) {
            Err(err) => {
                set_error(err);
                (null_mut(), EXIT_FAILURE)
            }
            Ok(Some(query)) => (box_ptr!(query), EXIT_SUCCESS),
            Ok(None) => (null_mut(), EXIT_SUCCESS),
        }
    });

    if let Ok((ret_query, ret_code)) = result {
        unsafe { *query.as_mut() = ret_query };
        ret_code
    } else {
        set_error(types::OperationalError::Unknown.into());
        unsafe { *query.as_mut() = null_mut() };
        EXIT_FAILURE
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
            EXIT_FAILURE
        } else {
            EXIT_SUCCESS
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
    let result = catch_unwind(|| {
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
                    Ok(_) => 1,
                    Err(e) => {
                        set_error(e);
                        0
                    }
                },
                Ok(_) => {
                    set_error(
                        types::RuntimeError::Serialization {
                            msg: "received bad command".to_string(),
                        }
                        .into(),
                    );
                    0
                }
                Err(e) => {
                    set_error(types::RuntimeError::Serialization { msg: e.to_string() }.into());
                    0
                }
            }
        } else {
            0
        }
    });
    match result {
        Ok(r) => r,
        Err(_) => {
            set_error(types::OperationalError::Unknown.into());
            0
        }
    }
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
                    return EXIT_FAILURE;
                }
            }
        }
        match polar.external_call_result(query, call_id, term) {
            Ok(_) => EXIT_SUCCESS,
            Err(e) => {
                set_error(e);
                EXIT_FAILURE
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
        let result = result != EXIT_FAILURE;
        polar.external_question_result(query, call_id, result);
        EXIT_SUCCESS
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
            return EXIT_FAILURE;
        }
        unsafe { CString::from_raw(s) };
        EXIT_SUCCESS
    })
}

/// Recovers the original boxed version of `polar` so that
/// it can be properly freed
#[no_mangle]
pub extern "C" fn polar_free(polar: *mut Polar) -> i32 {
    ffi_try!({
        std::mem::drop(unsafe { Box::from_raw(polar) });
        EXIT_SUCCESS
    })
}

/// Recovers the original boxed version of `query` so that
/// it can be properly freed
#[no_mangle]
pub extern "C" fn query_free(query: *mut Query) -> i32 {
    ffi_try!({
        std::mem::drop(unsafe { Box::from_raw(query) });
        EXIT_SUCCESS
    })
}

/// Free `load` created by `polar_new_load`.
#[no_mangle]
pub extern "C" fn load_free(load: *mut Load) -> i32 {
    ffi_try!({
        std::mem::drop(unsafe { Box::from_raw(load) });
        EXIT_SUCCESS
    })
}

//
// #[no_mangle]
// pub extern "C" fn polar_external_result(
//     polar_ptr: *mut Polar,
//     query_ptr: *mut Query,
//     result: *const c_char,
// ) {
//     let polar = unsafe { &mut *polar_ptr };
//     let mut query = unsafe { &mut *query_ptr };
//     let cs = unsafe { CStr::from_ptr(result) };
//     let s = cs.to_str().expect("to_str() failed");
//     polar.external_result(&mut query, s.to_owned());
// }
