//! Captcha feature.

use std::time::Duration as StdDuration;

use time::Duration;

/// Default length of the generated captcha code.
pub const DEFAULT_LENGTH: usize = 5;

/// Default duration before the captcha expires.
pub const DEFAULT_DURATION: Duration = Duration::minutes(5);

/// Duration before the member is kicked for not completing the verification.
pub const KICK_AFTER: StdDuration = StdDuration::from_secs(10);

/// Maximum number of regenerations of the captcha code.
pub const MAX_RETRY: u8 = 2;
