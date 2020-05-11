#[macro_use]
pub mod macros;

//mod parser;
mod parser;
mod polar;
pub mod types;
mod vm;

pub use self::polar::{Polar, Query};

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use serde_json;

// @TODO: Have a way to return errors, don't do any of these panics, that's gonna
// be real bad.

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

#[no_mangle]
pub extern "C" fn polar_new() -> *mut Polar {
    box_ptr!(Polar::new())
}

#[no_mangle]
pub extern "C" fn polar_load_str(polar_ptr: *mut Polar, src: *const c_char) {
    let polar = unsafe { ffi_ref!(polar_ptr) };
    let s = unsafe { ffi_string!(src) };
    polar.load_str(&s);
}

#[no_mangle]
pub extern "C" fn polar_new_query_from_predicate(
    polar_ptr: *mut Polar,
    query_pred: *const c_char,
) -> *mut Query {
    let polar = unsafe { ffi_ref!(polar_ptr) };
    let s = unsafe { ffi_string!(query_pred) };
    let predicate: types::Predicate = serde_json::from_str(&s).unwrap();

    box_ptr!(polar.new_query_from_predicate(predicate))
}

#[no_mangle]
pub extern "C" fn polar_new_query(polar_ptr: *mut Polar, query_str: *const c_char) -> *mut Query {
    let polar = unsafe { ffi_ref!(polar_ptr) };
    let s = unsafe { ffi_string!(query_str) };
    box_ptr!(polar.new_query(&s))
}

#[no_mangle]
pub extern "C" fn polar_query(polar_ptr: *mut Polar, query_ptr: *mut Query) -> *const c_char {
    let polar = unsafe { ffi_ref!(polar_ptr) };
    let query = unsafe { ffi_ref!(query_ptr) };
    let event = polar.query(query);
    // eprintln!("event: {:?}", event);
    let event_json = serde_json::to_string(&event).unwrap();
    CString::new(event_json)
        .expect("JSON should not contain any 0 bytes")
        .into_raw()
}

/// Required to free strings properly
#[no_mangle]
pub extern "C" fn string_free(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    unsafe { CString::from_raw(s) };
}

/// Recovers the original boxed version of `polar` so that
/// it can be properly freed
#[no_mangle]
pub extern "C" fn polar_free(polar: *mut Polar) {
    let _polar = unsafe { Box::from_raw(polar) };
}

/// Recovers the original boxed version of `query` so that
/// it can be properly freed
#[no_mangle]
pub extern "C" fn query_free(query: *mut Query) {
    let _query = unsafe { Box::from_raw(query) };
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
