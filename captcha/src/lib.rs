//! # Captcha generator
//!
//! This library contains the captcha image generator used by RaidProtect. The
//! generated [`RgbImage`] can be converted to any relevant image format.

pub mod code;

use image::{imageops::overlay, GrayAlphaImage, LumaA};
use imageproc::{
    drawing,
    geometric_transformations::{self, Interpolation, Projection},
    noise,
};
use once_cell::sync::Lazy;
use rand::{rngs::ThreadRng, seq::SliceRandom, Rng};
use rusttype::{Font, Scale};

/// Font used for the captcha generation.
///
/// The font is part of the GNU FreeFont family and licensed under GNU GPL v3.
/// See <https://www.gnu.org/software/freefont/>.
pub static FONT: Lazy<Font<'static>> =
    Lazy::new(|| Font::try_from_bytes(include_bytes!("../include/FreeMonoBold.ttf")).unwrap());

/// Height of the generated image
pub const IMAGE_HEIGHT: u32 = 150;

/// Height of a generated letter.
pub const LETTER_HEIGHT: u32 = 100;

/// Width of a generated letter.
pub const LETTER_WIDTH: u32 = 100;

/// Generate a new captcha image with the provided code.
pub fn generate_captcha(code: String) -> GrayAlphaImage {
    let image_width = (code.len() as u32 * LETTER_WIDTH) + 40;
    let mut image = GrayAlphaImage::new(image_width, IMAGE_HEIGHT);
    let mut rng = rand::thread_rng();

    // Draw letters
    for (index, letter) in code.char_indices() {
        let x = (index as u32 * LETTER_WIDTH) + 20;
        let y = rng.gen_range(0..70);

        let letter_image = generate_letter(letter, &mut rng);
        overlay(&mut image, &letter_image, x as i64, y);
    }

    // Add noise to the image
    noise::gaussian_noise_mut(&mut image, 80.0, 40.0, rng.gen_range(0..u64::MAX));

    image
}

/// Generate a captcha letter.
fn generate_letter(letter: char, rng: &mut ThreadRng) -> GrayAlphaImage {
    let mut image = GrayAlphaImage::new(LETTER_WIDTH, LETTER_HEIGHT);

    drawing::draw_text_mut(
        &mut image,
        LumaA([0, 255]),
        0,
        -20,
        Scale::uniform(120.0),
        &FONT,
        &letter.to_uppercase().to_string(),
    );

    letter_transform(&image, rng)
}

/// Applies a random transformation on the letter.
///
/// A projection is calculated with a randomization of the found image corners
/// coordinates.
fn letter_transform(image: &GrayAlphaImage, rng: &mut ThreadRng) -> GrayAlphaImage {
    let (width, height) = (image.dimensions().0 as f32, image.dimensions().1 as f32);

    // Choose which corners to transform.
    //
    // To avoid the letter to be unreadable, only two randomly chosen corners
    // are transformed.
    let mut corners = [true, true, false, false];
    corners.shuffle(rng);

    // Calculate new corners coordinates
    //
    // This code is ugly, but it works -- refactor it if you want.
    let mut gen_range = || rng.gen_range(15.0..35.0);

    let top_left_init = (0.0, 0.0);
    let top_right_init = (width, 0.0);
    let bottom_left_init = (0.0, height);
    let bottom_right_init = (width, height);

    let top_left = if corners[0] {
        (gen_range(), gen_range())
    } else {
        top_left_init
    };
    let top_right = if corners[1] {
        (width - gen_range(), gen_range())
    } else {
        top_right_init
    };
    let bottom_left = if corners[2] {
        (gen_range(), height - gen_range())
    } else {
        bottom_left_init
    };
    let bottom_right = if corners[3] {
        (width - gen_range(), height - gen_range())
    } else {
        bottom_right_init
    };

    // Calculate the projection of the image corners
    let projection = Projection::from_control_points(
        [
            top_left_init,
            top_right_init,
            bottom_left_init,
            bottom_right_init,
        ],
        [top_left, top_right, bottom_left, bottom_right],
    );

    // Apply the transformation
    if let Some(projection) = &projection {
        geometric_transformations::warp(image, projection, Interpolation::Bicubic, LumaA([0, 0]))
    } else {
        image.clone()
    }
}
