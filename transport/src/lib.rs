//! Protocol used to communicate between services.
//!
//! This crate contains types used to communicate between services.
//! Communication is based on a TCP connection with [`remoc`] channels.

pub mod cache;
pub mod client;
pub mod model;

pub use remoc; // Expose remoc to ensure the same default codec is used
