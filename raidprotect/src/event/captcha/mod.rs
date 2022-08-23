//! Captcha event processing.
//!
//! This module export functions used to process captcha-related events. The
//! following events are handled:
//!
//! - `MemberAdd`: when a member joins the server, the unverified role is added.

mod member_add;

pub use member_add::member_add;
