mod util;
use ed25519::signature::{Result, Signature};
use ed25519_dalek::{PublicKey, SecretKey};
use sha2::Sha512;
use util::byte_array_to_string;
pub struct Captcha {
    public_key: PublicKey,
    seed: Seed,
    image: CaptchaImage,
}

pub struct Seed(Box<[u8; 32]>);
pub struct CaptchaImage(Box<[u8; 32]>);

impl Captcha {
    pub fn new(captcha: [u8; 189]) -> Result<Self> {
        let public_key = PublicKey::from_bytes(&captcha[0..32])?;
        let seed = Seed(Box::new(captcha[32..64].try_into().expect("Seed should be 32 bytes long")));
        let image = CaptchaImage(Box::new(captcha[64..].try_into().expect("Image should be 189 bytes long")));

        Ok(Captcha {
            public_key,
            seed,
            image
        })
    }

    pub fn try_solve(&self, answer: &str) -> Option<SecretKey> {
        todo!()
    }
}

