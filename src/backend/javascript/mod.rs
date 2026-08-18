//! ECMAScript module emission from typed high-level intermediate representation.

mod ast;
mod emit;
mod lower;
mod optimize;
mod printer;

pub use emit::javascript_text;
