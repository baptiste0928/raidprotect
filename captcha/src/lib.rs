//! # Captcha generator
//!
//! This library contains the captcha image generator used by RaidProtect. The
//! generated [`RgbImage`] can be converted to any relevant image format.

pub mod code;

use std::io::Cursor;

use image::{
    imageops::overlay, DynamicImage, GrayAlphaImage, GrayImage, ImageError, ImageOutputFormat,
    LumaA, Pixel,
};
use imageproc::{
    drawing,
    geometric_transformations::{self, Interpolation, Projection},
};
use once_cell::sync::Lazy;
use rand::{rngs::ThreadRng, seq::SliceRandom, Rng};
use rusttype::{Font, Scale};

/// Font used for the captcha generation.
///
/// The font is part of the GNU FreeFont family and licensed under GNU GPL v3.
/// See <https://www.gnu.org/software/freefont/>.
static FONT: Lazy<Font<'static>> =
    Lazy::new(|| Font::try_from_bytes(include_bytes!("../include/FreeMonoBold.ttf")).unwrap());

const IMAGE_HEIGHT: u32 = 150;
const LETTER_HEIGHT: u32 = 100;
const LETTER_WIDTH: u32 = 100;

/// Generate a new captcha image with the provided code.
pub fn generate_captcha(code: &str) -> GrayImage {
    let image_width = (code.len() as u32 * LETTER_WIDTH) + 40;
    let mut image = GrayAlphaImage::from_pixel(image_width, IMAGE_HEIGHT, LumaA([255, 255]));
    let mut rng = rand::thread_rng();

    for (index, letter) in code.char_indices() {
        let x = (index as u32 * LETTER_WIDTH) + 20;
        let y = rng.gen_range(0..70);

        let letter_image = generate_letter(letter, &mut rng);
        overlay(&mut image, &letter_image, x as i64, y);
    }

    image_noise(&mut image, &mut rng);

    DynamicImage::ImageLumaA8(image).to_luma8()
}

/// Generate a new captcha with the provided code and encode it as png.
pub fn generate_captcha_png(code: &str) -> Result<Vec<u8>, ImageError> {
    let image = generate_captcha(code);
    let mut buffer = Cursor::new(Vec::new());

    image.write_to(&mut buffer, ImageOutputFormat::Png)?;

    Ok(buffer.into_inner())
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

    letter_transform(image, rng)
}

/// Applies a random transformation on the letter.
///
/// A projection is calculated with a randomization of the found image corners
/// coordinates.
fn letter_transform(image: GrayAlphaImage, rng: &mut ThreadRng) -> GrayAlphaImage {
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
        geometric_transformations::warp(&image, projection, Interpolation::Bicubic, LumaA([0, 0]))
    } else {
        image
    }
}

/// Add noise to the image.
fn image_noise(image: &mut GrayAlphaImage, rng: &mut ThreadRng) {
    for pixel in image.pixels_mut() {
        let noise = rng.gen_range(0..255);

        pixel.blend(&LumaA([noise, 128]));
    }
}
