//! Inspect Polar code to extract information about
//! rules, terms, etc.

mod rules;
mod terms;

pub use rules::{get_rule_information, RuleInfo};
pub use terms::{get_term_information, TermInfo};
