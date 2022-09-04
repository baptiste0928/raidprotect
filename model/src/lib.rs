//! # RaidProtect models
//!
//! This crate contains the models used to store the data shared by the
//! raidprotect components, such as the database or cache models. It also
//! exports utilities to interact with the data.
//!
//! Most models derive the [`Serialize`] and [`Deserialize`] traits to convert
//! them into the format used by the database. Some fields use a custom
//! serializer/deserializer targeted to a specific format, and thus shouldn't be
//! used with other formats.
//!
//! [`Serialize`]: ::serde::Serialize
//! [`Deserialize`]: ::serde::Deserialize

mod serde;

pub mod cache;
pub mod config;
pub mod database;
