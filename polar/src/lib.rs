#[macro_use]
pub mod macros;

#[cfg(test)]
#[macro_use]
extern crate maplit;

#[cfg(feature = "tui_")]
pub mod cli;
mod formatting;
mod lexer;
pub mod parser;
mod polar;
mod rewrites;
pub mod types;
mod vm;

pub use self::polar::{Load, Polar, Query};
pub use self::vm::DebugInfo;
pub use formatting::ToPolarString;

use std::cell::RefCell;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::panic::catch_unwind;

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
            let error_json = serde_json::to_string(&e).unwrap();
            CString::new(error_json)
                .expect("JSON should not contain any 0 bytes")
                .into_raw()
        } else {
            null()
        }
    });
    match result {
        Ok(r) => r,
        Err(_) => {
            set_error(types::OperationalError::Unknown.into());
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
            set_error(types::OperationalError::Unknown.into());
            null_mut()
        }
    }
}

/// Create a new Load struct from a load string.
///
/// Returns: A null ptr on error, otherwise a Load struct (must be freed by caller).
#[no_mangle]
pub extern "C" fn polar_new_load(polar_ptr: *mut Polar, src: *const c_char) -> *mut Load {
    let result = catch_unwind(|| {
        let polar = unsafe { ffi_ref!(polar_ptr) };
        let query_str = unsafe { ffi_string!(src) };
        match polar.new_load(&query_str) {
            Err(err) => {
                set_error(err);
                null_mut()
            }
            Ok(load) => box_ptr!(load),
        }
    });

    match result {
        Ok(r) => r,
        Err(_) => {
            set_error(types::OperationalError::Unknown.into());
            null_mut()
        }
    }
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
        return 1;
    };

    let result = catch_unwind(|| {
        let polar = unsafe { ffi_ref!(polar_ptr) };
        let load = unsafe { ffi_ref!(load) };
        match polar.load(load) {
            Err(err) => {
                set_error(err);
                (null_mut(), 1)
            }
            Ok(Some(query)) => (box_ptr!(query), 0),
            Ok(None) => (null_mut(), 0),
        }
    });

    match result {
        Ok((ret_query, ret_code)) => {
            unsafe { *query.as_mut() = ret_query };
            ret_code
        }
        Err(_) => {
            set_error(types::OperationalError::Unknown.into());
            unsafe { *query.as_mut() = null_mut() };
            1
        }
    }
}

#[no_mangle]
pub extern "C" fn polar_query_from_repl(polar_ptr: *mut Polar) -> *mut Query {
    let result = catch_unwind(|| {
        let polar = unsafe { ffi_ref!(polar_ptr) };
        match polar.new_query_from_repl() {
            Ok(query) => box_ptr!(query),
            Err(e) => {
                set_error(types::RuntimeError::Serialization { msg: e.to_string() }.into());
                null_mut()
            }
        }
    });
    match result {
        Ok(r) => r,
        Err(_) => {
            set_error(types::OperationalError::Unknown.into());
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
            set_error(types::OperationalError::Unknown.into());
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
                set_error(types::RuntimeError::Serialization { msg: e.to_string() }.into());
                null_mut()
            }
        }
    });
    match result {
        Ok(r) => r,
        Err(_) => {
            set_error(types::OperationalError::Unknown.into());
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
            set_error(types::OperationalError::Unknown.into());
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
            set_error(types::OperationalError::Unknown.into());
            null()
        }
    }
}

/// Register class with `class_name` with polar object.
///
/// Returns: 1 on success, 0 on error.
#[no_mangle]
pub extern "C" fn polar_register_external_class(
    polar_ptr: *mut Polar,
    class_name: *const c_char
) -> i32 {
    let result = catch_unwind(|| {
        let polar = unsafe { ffi_ref!(polar_ptr) };
        let class_name = unsafe { ffi_string!(class_name) };

        match polar.register_external_class(&class_name) {
            Ok(()) => 1,
            Err(e) => {
                set_error(e);
                0
            }
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
    let result = catch_unwind(|| {
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
                    return 0;
                }
            }
        }
        match polar.external_call_result(query, call_id, term) {
            Ok(_) => 1,
            Err(e) => {
                set_error(e);
                0
            }
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
pub extern "C" fn polar_external_question_result(
    polar_ptr: *mut Polar,
    query_ptr: *mut Query,
    call_id: u64,
    result: i32,
) -> i32 {
    let result = catch_unwind(|| {
        let polar = unsafe { ffi_ref!(polar_ptr) };
        let query = unsafe { ffi_ref!(query_ptr) };
        let result = if let 0 = result { false } else { true };
        polar.external_question_result(query, call_id, result);
        1
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
pub extern "C" fn polar_get_external_id(polar_ptr: *mut Polar) -> u64 {
    let result = catch_unwind(|| {
        let polar = unsafe { ffi_ref!(polar_ptr) };
        polar.get_external_id()
    });
    match result {
        Ok(r) => r,
        Err(_) => {
            set_error(types::OperationalError::Unknown.into());
            0
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
        Ok(_) => (),
        Err(_) => {
            set_error(types::OperationalError::Unknown.into());
        }
    }
}

/// Recovers the original boxed version of `polar` so that
/// it can be properly freed
#[no_mangle]
pub extern "C" fn polar_free(polar: *mut Polar) -> i32 {
    let result = catch_unwind(|| {
        std::mem::drop(unsafe { Box::from_raw(polar) });
    });

    match result {
        Ok(_) => 0,
        Err(_) => {
            set_error(types::OperationalError::Unknown.into());
            1
        }
    }
}

/// Recovers the original boxed version of `query` so that
/// it can be properly freed
#[no_mangle]
pub extern "C" fn query_free(query: *mut Query) -> i32 {
    let result = catch_unwind(|| {
        std::mem::drop(unsafe { Box::from_raw(query) });
    });

    match result {
        Ok(_) => 0,
        Err(_) => {
            set_error(types::OperationalError::Unknown.into());
            1
        }
    }
}

/// Free `load` created by `polar_new_load`.
#[no_mangle]
pub extern "C" fn load_free(load: *mut Load) -> i32 {
    let result = catch_unwind(|| {
        std::mem::drop(unsafe { Box::from_raw(load) });
    });

    match result {
        Ok(_) => 0,
        Err(_) => {
            set_error(types::OperationalError::Unknown.into());
            1
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
