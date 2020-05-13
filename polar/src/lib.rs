#[macro_use]
pub mod macros;

//mod parser;
mod parser;
mod polar;
pub mod types;
mod vm;

pub use self::polar::{Polar, Query};

use std::cell::RefCell;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::panic::catch_unwind;

use serde_json;
use std::ptr::{null, null_mut};

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

thread_local! {
    static LAST_ERROR: RefCell<Option<Box<types::PolarError>>> = RefCell::new(None);
}

fn set_error(e: types::PolarError) {
    LAST_ERROR.with(|prev| *prev.borrow_mut() = Some(Box::new(e)))
}

#[no_mangle]
pub extern "C" fn polar_get_error() -> *const c_char {
    let result = catch_unwind(|| {
        let err = LAST_ERROR.with(|prev| prev.borrow_mut().take());
        if let Some(e) = err {
            CString::new(e.to_string())
                .expect("Error message should not contain any 0 bytes")
                .into_raw()
        } else {
            null()
        }
    });
    match result {
        Ok(r) => r,
        Err(_) => {
            set_error(types::PolarError::Unknown);
            null()
        }
    }
}

#[no_mangle]
pub extern "C" fn polar_new() -> *mut Polar {
    let result = catch_unwind(|| box_ptr!(Polar::new()));
    match result {
        Ok(r) => r,
        Err(_) => {
            set_error(types::PolarError::Unknown);
            null_mut()
        }
    }
}

#[no_mangle]
/// Bools aren't portable, 0 means error 1 means success.
pub extern "C" fn polar_load_str(polar_ptr: *mut Polar, src: *const c_char) -> i32 {
    let result = catch_unwind(|| {
        let polar = unsafe { ffi_ref!(polar_ptr) };
        let s = unsafe { ffi_string!(src) };
        if let Err(e) = polar.load_str(&s) {
            set_error(e);
            0
        } else {
            1
        }
    });
    match result {
        Ok(r) => r,
        Err(_) => {
            set_error(types::PolarError::Unknown);
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn polar_new_query_from_term(
    polar_ptr: *mut Polar,
    query_term: *const c_char,
) -> *mut Query {
    let result = catch_unwind(|| {
        let polar = unsafe { ffi_ref!(polar_ptr) };
        let s = unsafe { ffi_string!(query_term) };
        let term = serde_json::from_str(&s);
        match term {
            Ok(term) => box_ptr!(polar.new_query_from_term(term)),
            Err(e) => {
                set_error(types::PolarError::Serialization(e.to_string()));
                null_mut()
            }
        }
    });
    match result {
        Ok(r) => r,
        Err(_) => {
            set_error(types::PolarError::Unknown);
            null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn polar_new_query(polar_ptr: *mut Polar, query_str: *const c_char) -> *mut Query {
    let result = catch_unwind(|| {
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
    });
    match result {
        Ok(r) => r,
        Err(_) => {
            set_error(types::PolarError::Unknown);
            null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn polar_query(polar_ptr: *mut Polar, query_ptr: *mut Query) -> *const c_char {
    let result = catch_unwind(|| {
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
    });
    match result {
        Ok(r) => r,
        Err(_) => {
            set_error(types::PolarError::Unknown);
            null()
        }
    }
}

/// Required to free strings properly
#[no_mangle]
pub extern "C" fn string_free(s: *mut c_char) {
    let result = catch_unwind(|| {
        if s.is_null() {
            return;
        }
        unsafe { CString::from_raw(s) };
    });
    match result {
        Ok(r) => (),
        Err(_) => {
            set_error(types::PolarError::Unknown);
        }
    }
}

/// Recovers the original boxed version of `polar` so that
/// it can be properly freed
#[no_mangle]
pub extern "C" fn polar_free(polar: *mut Polar) {
    let result = catch_unwind(|| {
        let _polar = unsafe { Box::from_raw(polar) };
    });
    match result {
        Ok(r) => (),
        Err(_) => {
            set_error(types::PolarError::Unknown);
        }
    }
}

/// Recovers the original boxed version of `query` so that
/// it can be properly freed
#[no_mangle]
pub extern "C" fn query_free(query: *mut Query) {
    let result = catch_unwind(|| {
        let _query = unsafe { Box::from_raw(query) };
    });
    match result {
        Ok(r) => (),
        Err(_) => {
            set_error(types::PolarError::Unknown);
        }
    }
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
