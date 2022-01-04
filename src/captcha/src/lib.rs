mod util;

use std::num::ParseIntError;

use ed25519::{signature::Result as SigResult, signature::Verifier, Error, Signature};
use ed25519_dalek::{
    ExpandedSecretKey, PublicKey, EXPANDED_SECRET_KEY_LENGTH, PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH,
};
use image::{RgbImage, Rgb};
use sha2::{Digest, Sha512};
use util::{byte_array_to_hex_string, hex_string_to_byte_array};
pub struct InvalidSignature;
impl From<ParseIntError> for InvalidSignature {
    fn from(_: ParseIntError) -> Self {
        InvalidSignature
    }
}

impl From<Error> for InvalidSignature {
    fn from(_: Error) -> Self {
        InvalidSignature
    }
}

const SEED_LEN: usize = PUBLIC_KEY_LENGTH;
const IMAGE_LEN: usize = 125;
const CAPTCHA_LEN: usize = 189;

pub struct Captcha {
    public_key: PublicKey,
    seed: Seed,
    image: CaptchaImage,
}

pub type Seed = [u8; SEED_LEN];
pub type CaptchaImage = [u8; IMAGE_LEN];

impl Captcha {
    pub fn new(captcha: [u8; CAPTCHA_LEN]) -> Result<Self, Error> {
        let public_key = PublicKey::from_bytes(&captcha[0..32])?;
        let seed = captcha[32..64]
            .try_into()
            .expect("Seed should be 32 bytes long");

        let image: [u8; IMAGE_LEN] = captcha[64..189]
            .try_into()
            .expect("image is too long or short");

        Ok(Captcha {
            public_key,
            seed,
            image,
        })
    }

    /// Returns a signature of the post if the captcha guess is correct
    /// # Arguments
    /// * `answer` - A string slice with the captcha answer
    /// * `post` - A post which has to be signed
    pub fn try_sign(&self, answer: &str, post: &str) -> Option<String> {
        let secret_key = self.decrypt_seed(answer);
        if let Err(e) = secret_key {
            println!("Error decrypting secret key: {:?}", e);
            return None;
        }
        let secret_key = secret_key.ok()?;
        self.verify_key(&secret_key).ok()?;

        let signature = secret_key.sign(post.as_bytes(), &self.public_key);
        Some(byte_array_to_hex_string(&signature.to_bytes()).unwrap())
    }

    pub fn signature_correct(&self, post: &str, signature: &str) -> Result<(), InvalidSignature> {
        let signature_bytes = hex_string_to_byte_array(signature)?;
        let signature = Signature::from_bytes(&signature_bytes)?;
        self.public_key.verify(post.as_bytes(), &signature)?;

        Ok(())
    }
    pub fn build_image(&self) -> RgbImage {
        const WIDTH: u32 = 50;
        const HEIGHT: u32 = 20;
        const BLACK: Rgb<u8> = Rgb([0, 0, 0]);
        const WHITE: Rgb<u8> = Rgb([255, 255, 255]);

        let mut img = RgbImage::new(WIDTH, HEIGHT);
        let mut bii = 0;
        let mut byi = 0;

        let (width, height) = img.dimensions();

        for x in 0..width {
            for y in 0..height {
                let color = if (self.image[byi] & (1 << bii)) != 0 {
                    BLACK
                } else {
                    WHITE
                };

                bii += 1;

                if bii >= 8 {
                    bii = 0;
                    byi += 1;
                }

                img.put_pixel(x, y, color);
            }
        }

        img
    }

    fn verify_key(&self, secret_key: &ExpandedSecretKey) -> Result<(), Error> {
        let test_message = [1u8];
        let signature = secret_key.sign(&test_message, &self.public_key);
        self.public_key
            .verify(&test_message, &signature)
            .or_else(|op| {
                println!("Error verifying decrypted secret: {:?}", op);
                Err(op)
            })
    }

    fn decrypt_seed(&self, answer: &str) -> SigResult<ExpandedSecretKey> {
        let public_key = byte_array_to_hex_string(self.public_key.as_bytes()).unwrap();
        let combined = format!("{}{}", answer, public_key);
        let mut hasher = Sha512::new();
        hasher.update(combined.as_bytes());
        let hash = hasher.finalize();

        let mut decrypted_seed = [0u8; SECRET_KEY_LENGTH];
        for (i, byte) in self.seed.iter().enumerate() {
            decrypted_seed[i] = *byte ^ hash[i & 63];
        }

        let secret_key = ed25519_dalek::SecretKey::from_bytes(&decrypted_seed)?;
        let expanded = ExpandedSecretKey::from(&secret_key);
        Ok(expanded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::*;
    use std::io::{Read, Seek, SeekFrom};
    const CAPTCHA_OFFSET: usize = 13;
    const CAPTCHA_ANSWER: &str = "bavzr";
    //todo signature tests

    #[test]
    fn generated_signature_is_correct() {
        let post = "super post!";
        let captcha_file = get_captcha_file();
        let captcha = read_captcha(CAPTCHA_OFFSET, captcha_file);
        let signature = captcha.try_sign(CAPTCHA_ANSWER, &post).unwrap();
        let verification = captcha.signature_correct(&post, &signature);

        assert!(verification.is_ok())
    }

    #[test]
    fn generated_signature_none_if_answer_wrong() {
        let post = "ya pirog";
        let captcha_file = get_captcha_file();
        let captcha = read_captcha(CAPTCHA_OFFSET, captcha_file);
        let fake_answer = "pirog";
        let signature = captcha.try_sign(fake_answer, post);

        assert!(signature.is_none())
    }

    fn read_captcha(offset: usize, mut file: impl Read + Seek) -> Captcha {
        let mut buffer = [0; CAPTCHA_LEN];
        file.seek(SeekFrom::Start((offset * CAPTCHA_LEN).try_into().unwrap()))
            .unwrap();
        file.read_exact(&mut buffer).unwrap();

        Captcha::new(buffer).unwrap()
    }

    fn get_captcha_file() -> File {
        File::open("captcha.nbc").unwrap()
    }
}
