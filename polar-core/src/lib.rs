#[cfg(test)]
#[macro_use]
extern crate maplit;

mod debugger;
pub mod error;
pub mod formatting;
mod lexer;
#[macro_use]
pub mod macros;
pub mod events;
mod kb;
mod messages;
mod numerics;
pub mod parser;
pub mod polar;
mod rewrites;
mod rules;
mod sources;
pub mod terms;
pub mod traces;
pub mod types;
mod vm;
mod warnings;
