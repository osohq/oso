//! Language diagnostics: e.g. lints, warnings, and errors

mod errors;
mod unused_rules;

pub use errors::find_parse_errors;
pub use unused_rules::find_unused_rules;
