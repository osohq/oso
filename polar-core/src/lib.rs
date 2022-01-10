#![allow(clippy::vec_init_then_push)]

#[cfg(test)]
#[macro_use]
extern crate maplit;

#[macro_use]
pub mod macros;

mod counter;
pub mod data_filtering;
pub mod diagnostic;
pub mod error;
pub mod filter;
mod folder;
pub mod formatting;
pub mod kb;
mod lexer;
pub mod messages;
pub mod normalize;
mod numerics;
pub mod parser;
pub mod polar;
pub mod query;
pub mod resource_block;
pub mod rules;
pub mod sources;
pub mod terms;
pub mod traces;
mod validations;
mod visitor;
pub mod warning;

// TODO: remove

use std::collections::HashMap;
use terms::{Symbol, Term};

pub type Bindings = HashMap<Symbol, Term>;

pub struct ResultEvent {
    bindings: Bindings,
}

impl ResultEvent {
    fn new(bindings: Bindings) -> Self {
        Self { bindings }
    }
}

pub use query::Query;
