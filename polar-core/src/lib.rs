#![allow(clippy::vec_init_then_push)]

#[cfg(test)]
#[macro_use]
extern crate maplit;

#[macro_use]
pub mod macros;

mod bindings;
mod counter;
mod debugger;
pub mod error;
pub mod events;
mod folder;
pub mod formatting;
mod inverter;
pub mod kb;
mod lexer;
pub mod messages;
mod numerics;
pub mod parser;
mod partial;
pub mod polar;
mod rewrites;
mod roles_validation;
pub mod rules;
mod runnable;
mod sources;
pub mod terms;
pub mod traces;
mod visitor;
mod vm;
mod warnings;
