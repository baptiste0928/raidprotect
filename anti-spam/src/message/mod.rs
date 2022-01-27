//! Message parsing.
//!
//! This module is responsible of parsing incoming messages. Parsing consists
//! of extracting message features and some contextual data in the message that
//! will be then used to detect spam.
//!
//! ## Feature extraction
//! Message content is analyzed to detected common characteristics (or features).
//!
//! The following characteristics are detected:
//! - URLs, with detection of server invite links.
//! - Attachements, like images or files. Image links are handled as attachements.
//! - User, roles and special (@everyone and @here) mentions.

mod model;
mod parse;
