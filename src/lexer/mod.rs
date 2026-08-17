//! Layout-aware lexical analysis for the parser.
//!
//! Raw token recognition stays separate from indentation and logical-newline state.

mod layout;
mod raw;
mod token;

pub use layout::lex;
pub use token::{Token, TokenKind};
