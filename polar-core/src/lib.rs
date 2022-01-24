#![allow(clippy::vec_init_then_push)]

#[cfg(test)]
#[macro_use]
extern crate maplit;

#[macro_use]
pub mod macros;

mod bindings;
mod constants;
mod counter;
pub mod data_filtering;
mod debugger;
pub mod diagnostic;
pub mod error;
pub mod events;
pub mod filter;
mod folder;
mod formatting;
mod inverter;
pub mod kb;
mod lexer;
pub mod messages;
pub mod normalize;
mod numerics;
pub mod parser;
mod partial;
pub mod polar;
pub mod query;
pub mod resource_block;
mod rewrites;
pub mod rules;
mod runnable;
pub mod sources;
pub mod terms;
pub mod traces;
mod validations;
mod visitor;
mod vm;
pub mod warning;

pub use lexer::loc_to_pos;
