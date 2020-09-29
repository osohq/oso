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
pub mod kb;
pub mod messages;
mod numerics;
pub mod parser;
mod partial;
pub mod polar;
mod runnable;
mod rewrites;
pub mod rules;
mod sources;
pub mod terms;
pub mod traces;
mod vm;
mod warnings;
