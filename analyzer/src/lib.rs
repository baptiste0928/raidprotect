//! # RaidProtect message analyzer
//!
//! This crate contains the message parser and analyzer used to detect user spam
//! or filter messages.

mod parser;

pub use parser::parse_message;
