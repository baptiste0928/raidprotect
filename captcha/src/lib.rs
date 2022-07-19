//! # Captcha generator
//!
//! This library contains the captcha image generator used by RaidProtect. The
//! generated [`RgbImage`] can be converted to any relevant image format.

pub mod code;

use image::RgbImage;

const IMAGE_HEIGHT: u32 = 150;
const IMAGE_WIDTH: u32 = 400;

/// Generate a new captcha image with the provided code.
pub fn generate_captcha(_code: String) -> RgbImage {
    RgbImage::new(IMAGE_WIDTH, IMAGE_HEIGHT)
}
