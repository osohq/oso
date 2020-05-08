//mod parser;
mod parser;
mod polar;
mod types;
mod vm;

pub use polar::{Polar, Query};

use std::ffi::{CStr, CString};
use std::mem::transmute;
use std::os::raw::c_char;

use serde_json;

static mut DATA: *const CString = 0 as *const CString;

// @TODO: Have a way to return errors, don't do any of these panics, that's gonna
// be real bad.
// #[no_mangle]
// pub extern "C" fn query_new_from_pred(query_pred: *const c_char) -> *mut Query {
//     let cs = unsafe { CStr::from_ptr(query_pred) };
//     let s = cs.to_str().expect("to_str() failed");
//     let predicate: types::Predicate = serde_json::from_str(s).unwrap();

//     let q = Box::new(Query::new_from_pred(predicate));
//     unsafe { transmute(q) }
// }

#[no_mangle]
pub extern "C" fn polar_new() -> *mut Polar {
    let p = Box::new(Polar::new());
    unsafe { transmute(p) }
}

#[no_mangle]
pub extern "C" fn polar_load_str(polar_ptr: *mut Polar, src: *const c_char) {
    let polar = unsafe { &mut *polar_ptr };
    let cs = unsafe { CStr::from_ptr(src) };
    let s = cs.to_str().expect("to_str() failed");
    polar.load_str(s);
}

#[no_mangle]
pub extern "C" fn polar_new_query_from_predicate(polar_ptr: *mut Polar, query_pred: *const c_char) -> *mut Query {
    let polar = unsafe { &mut *polar_ptr };
    let cs = unsafe { CStr::from_ptr(query_pred) };
    let s = cs.to_str().expect("to_str() failed");
    let predicate: types::Predicate = serde_json::from_str(s).unwrap();

    let q = Box::new(polar.new_query_from_predicate(predicate));
    unsafe { transmute(q) }
}

#[no_mangle]
pub extern "C" fn polar_new_query(polar_ptr: *mut Polar, query_str: *const c_char) -> *mut Query {
    let polar = unsafe { &mut *polar_ptr };
    let cs = unsafe { CStr::from_ptr(query_str) };
    let s = cs.to_str().expect("to_str() failed");
    let q = Box::new(polar.new_query(s));
    unsafe { transmute(q) }
}

#[no_mangle]
pub extern "C" fn polar_query(polar_ptr: *mut Polar, query_ptr: *mut Query) -> *const c_char {
    let polar = unsafe { &mut *polar_ptr };
    let query = unsafe { &mut *query_ptr };
    let event = polar.query(query);
    let event_json = serde_json::to_string(&event).unwrap();
    let boxed_json = Box::new(CString::new(event_json).unwrap());
    // @TODO: If there's something at ptr free it, so we don't leak stuff.
    unsafe {
        DATA = transmute(boxed_json);
        (&*DATA).as_ptr()
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
