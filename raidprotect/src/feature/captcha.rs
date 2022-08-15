//! Captcha feature.

use time::Duration;

/// Default length of the generated captcha code.
pub const DEFAULT_LENGTH: usize = 5;

/// Default duration before the captcha expires.
pub const DEFAULT_DURATION: Duration = Duration::minutes(5);
