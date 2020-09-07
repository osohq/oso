//! Builtin types supported in Polar

use polar_core::terms::{Symbol, Value};

use std::collections::HashMap;

use super::Class;

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
        .add_method("ends_with", |s: &String, pat: String| s.ends_with(&pat))
}

/// Returns the builtin types, the name, class, and instance
pub fn classes() -> Vec<(Symbol, Class)> {
    vec![
        (Symbol("Boolean".to_string()), boolean().erase_type()),
        (Symbol("Integer".to_string()), integer().erase_type()),
        (Symbol("Float".to_string()), float().erase_type()),
        (Symbol("List".to_string()), list().erase_type()),
        (Symbol("Dictionary".to_string()), dictionary().erase_type()),
        (Symbol("String".to_string()), string().erase_type()),
    ]
}
