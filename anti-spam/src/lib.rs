//! # RaidProtect anti-spam
//!
//! This crate contains the anti-spam engine of RaidProtect.
//!
//! ## Implementation
//! The anti-spam engine process and analyze messages sent by users to detect
//! whether they are spamming or not. Two different kinds of spam are detected:
//! user-level spam and channel-level spam, involving multiple accounts.

mod message;
