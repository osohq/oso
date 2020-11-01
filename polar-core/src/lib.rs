#[cfg(test)]
#[macro_use]
extern crate maplit;

mod debugger;
pub mod error;
pub mod formatting;
mod lexer;
#[macro_use]
pub mod macros;
mod counter;
pub mod events;
mod folder;
pub mod kb;
pub mod messages;
mod monad;
mod numerics;
pub mod parser;
mod partial;
pub mod polar;
mod rewrites;
pub mod rules;
mod runnable;
mod sources;
pub mod terms;
pub mod traces;
mod visitor;
mod vm;
mod warnings;
