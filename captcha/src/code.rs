//! Generation of random captcha codes.
//!
//! - [`random_code`] generates a random code using alphabetic ascii characters.
//! - [`random_human_code`] generates a random human-readable code using
//!   alphabetic ascii character.

use rand::{rngs::ThreadRng, Rng};

/// Generates a random code.
///
/// The generated code is a [`String`] of `len` random a-z ascii characters.
pub fn random_code(len: usize) -> String {
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz";

    let mut rng = rand::thread_rng();
    let mut code = String::with_capacity(len);

    for _ in 0..len {
        code.push(random_char(&mut rng, CHARSET));
    }

    code
}

/// Generates a random human-readable code.
///
/// The generated code is a 5 characters [`String`]. The used algorithm can
/// produce 2^16 different codes.
///
/// Adapted from [Proquints](https://arxiv.org/html/0901.4016).
pub fn random_human_code() -> String {
    const CONSONANTS: &[u8] = b"bdfghjklmnprstvz";
    const VOWELS: &[u8] = b"aiou";

    let mut rng = rand::thread_rng();
    let mut code = String::with_capacity(5);

    code.push(random_char(&mut rng, CONSONANTS));
    code.push(random_char(&mut rng, VOWELS));
    code.push(random_char(&mut rng, CONSONANTS));
    code.push(random_char(&mut rng, VOWELS));
    code.push(random_char(&mut rng, CONSONANTS));

    code
}

fn random_char(rng: &mut ThreadRng, charset: &[u8]) -> char {
    let index = rng.gen_range(0..charset.len());

    charset[index] as char
}

#[cfg(test)]
mod tests {
    use super::{random_code, random_human_code};

    #[test]
    fn test_random_code() {
        let code_1 = random_code(5);
        let code_2 = random_code(5);

        assert_eq!(code_1.len(), 5);
        assert_ne!(code_1, code_2);
    }

    #[test]
    fn test_random_human_code() {
        let code_1 = random_human_code();
        let code_2 = random_human_code();

        assert_eq!(code_1.len(), 5);
        assert_ne!(code_1, code_2);
    }
}
