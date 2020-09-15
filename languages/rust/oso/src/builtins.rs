//! Builtin types supported in Polar

use polar_core::terms::{Symbol, Value};

use std::collections::HashMap;

use crate::Class;

fn boolean() -> Class<bool> {
    Class::<bool>::with_default().name("Boolean")
}

fn integer() -> Class<i64> {
    Class::<i64>::with_default().name("Integer")
}

fn float() -> Class<f64> {
    Class::<f64>::with_default().name("Float")
}

fn list() -> Class<Vec<Value>> {
    Class::<Vec<Value>>::with_default().name("List")
}

fn dictionary() -> Class<HashMap<Symbol, Value>> {
    Class::<HashMap<Symbol, Value>>::with_default().name("Dictionary")
}

fn string() -> Class<String> {
    Class::<String>::with_default()
        .name("String")
        .add_method("ends_with", |s: &String, pat: String| s.ends_with(&pat))
}

/// Returns the builtin types, the name, class, and instance
pub fn classes() -> Vec<Class> {
    vec![
        boolean().erase_type(),
        integer().erase_type(),
        float().erase_type(),
        list().erase_type(),
        dictionary().erase_type(),
        string().erase_type(),
    ]
}
