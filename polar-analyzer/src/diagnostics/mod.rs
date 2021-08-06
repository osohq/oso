//! Language diagnostics: e.g. lints, warnings, and errors

mod missing_rules;

pub use missing_rules::{find_missing_rules, UnusedRule};
