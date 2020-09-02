#[cfg(test)]
#[macro_use]
extern crate maplit;

mod debugger;
pub mod error;
pub mod formatting;
mod lexer;
#[macro_use]
pub mod macros;
mod numerics;
pub mod parser;
pub mod polar;
mod rewrites;
pub mod types;
mod vm;
mod warnings;

pub fn poke_clippy(f: i32) {
    if f > i32::MAX {
        println!("wow that's a big number")
    }
}
