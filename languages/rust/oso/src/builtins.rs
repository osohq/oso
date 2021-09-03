//! Builtin types supported in Polar

use crate::PolarValue;
use std::collections::HashMap;

use crate::{Class, ClassBuilder};

fn boolean() -> ClassBuilder<bool> {
    ClassBuilder::<bool>::with_default()
        .with_equality_check()
        .name("Boolean")
}

fn integer() -> ClassBuilder<i64> {
    ClassBuilder::<i64>::with_default()
        .with_equality_check()
        .name("Integer")
}

fn float() -> ClassBuilder<f64> {
    ClassBuilder::<f64>::with_default()
        .with_equality_check()
        .name("Float")
}

fn list() -> ClassBuilder<Vec<PolarValue>> {
    ClassBuilder::<Vec<PolarValue>>::with_default()
        .with_equality_check()
        .name("List")
}

fn dictionary() -> ClassBuilder<HashMap<String, PolarValue>> {
    ClassBuilder::<HashMap<String, PolarValue>>::with_default()
        .with_equality_check()
        .name("Dictionary")
}

fn option() -> ClassBuilder<Option<PolarValue>> {
    ClassBuilder::<Option<PolarValue>>::with_default()
        .with_equality_check()
        .name("Option")
        .add_method("unwrap", |v: &Option<PolarValue>| v.clone().unwrap())
        .add_method("is_none", Option::is_none)
        .add_method("is_some", Option::is_some)
        .with_iter()
}

fn string() -> ClassBuilder<String> {
    ClassBuilder::<String>::with_default()
        .with_equality_check()
        .name("String")
        .add_method("len", |s: &String| s.len() as i64)
        .add_method("is_empty", String::is_empty)
        .add_method("is_char_boundary", |s: &String, index: i64| {
            s.is_char_boundary(index as usize)
        })
        .add_method("bytes", |s: &String| {
            s.bytes().map(|c| c as i64).collect::<Vec<i64>>()
        })
        .add_method("chars", |s: &String| {
            s.chars().map(|c| c.to_string()).collect::<Vec<String>>()
        })
        .add_method("char_indices", |s: &String| {
            s.char_indices()
                .map(|(i, c)| {
                    vec![
                        PolarValue::Integer(i as i64),
                        PolarValue::String(c.to_string()),
                    ]
                })
                .collect::<Vec<Vec<PolarValue>>>()
        })
        .add_method("split_whitespace", |s: &String| {
            s.split_whitespace()
                .map(|c| c.to_string())
                .collect::<Vec<String>>()
        })
        .add_method("lines", |s: &String| {
            s.lines().map(|c| c.to_string()).collect::<Vec<String>>()
        })
        .add_method("lines", |s: &String| {
            s.lines().map(|c| c.to_string()).collect::<Vec<String>>()
        })
        .add_method("contains", |s: &String, pat: String| s.contains(&pat))
        .add_method("starts_with", |s: &String, pat: String| s.starts_with(&pat))
        .add_method("ends_with", |s: &String, pat: String| s.ends_with(&pat))
        .add_method("find", |s: &String, pat: String| {
            s.find(&pat).map(|i| i as i64)
        })
        .add_method("rfind", |s: &String, pat: String| {
            s.rfind(&pat).map(|i| i as i64)
        })
        .add_method("split", |s: &String, pat: String| {
            s.split(&pat)
                .map(|i| i.to_string())
                .collect::<Vec<String>>()
        })
        .add_method("rsplit", |s: &String, pat: String| {
            s.rsplit(&pat)
                .map(|i| i.to_string())
                .collect::<Vec<String>>()
        })
        .add_method("split_terminator", |s: &String, pat: String| {
            s.split_terminator(&pat)
                .map(|i| i.to_string())
                .collect::<Vec<String>>()
        })
        .add_method("rsplit_terminator", |s: &String, pat: String| {
            s.rsplit_terminator(&pat)
                .map(|i| i.to_string())
                .collect::<Vec<String>>()
        })
        .add_method("splitn", |s: &String, n: i32, pat: String| {
            s.splitn(n as usize, &pat)
                .map(|i| i.to_string())
                .collect::<Vec<String>>()
        })
        .add_method("rsplitn", |s: &String, n: i32, pat: String| {
            s.rsplitn(n as usize, &pat)
                .map(|i| i.to_string())
                .collect::<Vec<String>>()
        })
        .add_method("matches", |s: &String, pat: String| {
            s.matches(&pat)
                .map(|i| i.to_string())
                .collect::<Vec<String>>()
        })
        .add_method("rmatches", |s: &String, pat: String| {
            s.rmatches(&pat)
                .map(|i| i.to_string())
                .collect::<Vec<String>>()
        })
        .add_method("match_indices", |s: &String, pat: String| {
            s.match_indices(&pat)
                .map(|(i, c)| {
                    vec![
                        PolarValue::Integer(i as i64),
                        PolarValue::String(c.to_string()),
                    ]
                })
                .collect::<Vec<Vec<PolarValue>>>()
        })
        .add_method("rmatch_indices", |s: &String, pat: String| {
            s.rmatch_indices(&pat)
                .map(|(i, c)| {
                    vec![
                        PolarValue::Integer(i as i64),
                        PolarValue::String(c.to_string()),
                    ]
                })
                .collect::<Vec<Vec<PolarValue>>>()
        })
        .add_method("trim", |s: &String| s.trim().to_string())
        .add_method("trim_start", |s: &String| s.trim_start().to_string())
        .add_method("trim_end", |s: &String| s.trim_end().to_string())
        .add_method("is_ascii", |s: &String| s.is_ascii())
        .add_method("to_lowercase", |s: &String| s.to_lowercase())
        .add_method("to_uppercase", |s: &String| s.to_uppercase())
        .add_method("repeat", |s: &String, n: i64| s.repeat(n as usize))
}

/// Returns the builtin types, the name, class, and instance
pub fn classes() -> Vec<Class> {
    vec![
        boolean().build(),
        integer().build(),
        float().build(),
        list().build(),
        dictionary().build(),
        string().build(),
        option().build(),
    ]
}
