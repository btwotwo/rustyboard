mod util;

use ed25519::Error;
use ed25519_dalek::{ExpandedSecretKey, PublicKey, Verifier, PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH};
use sha2::{Digest, Sha512};
use util::byte_array_to_string;
const SEED_LEN: usize = PUBLIC_KEY_LENGTH;
const IMAGE_LEN: usize = 125;

pub struct Captcha {
    public_key: PublicKey,
    seed: Seed,
    image: CaptchaImage,
}

pub struct Seed(Box<[u8; SEED_LEN]>);
pub struct CaptchaImage(Box<[u8; IMAGE_LEN]>);

impl Captcha {
    pub fn new(captcha: [u8; 189]) -> Result<Self, Error> {
        let public_key = PublicKey::from_bytes(&captcha[0..32])?;
        let seed = Seed(Box::new(
            captcha[32..64]
                .try_into()
                .expect("Seed should be 32 bytes long"),
        ));
        let image = CaptchaImage(Box::new(
            captcha[64..]
                .try_into()
                .expect("Image should be 189 bytes long"),
        ));

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
        let secret_key = self.decrypt_seed(answer)?;
        self.verify_key(&secret_key).ok()?;

        let signature = secret_key.sign(post.as_bytes(), &self.public_key);
        Some(byte_array_to_string(&signature.to_bytes()).unwrap())
    }

    fn verify_key(&self, secret_key: &ExpandedSecretKey) -> Result<(), Error> {
        let test_message = [1u8];
        let signature = secret_key.sign(&test_message, &self.public_key);
        self.public_key.verify(&test_message, &signature)
    }

    fn decrypt_seed(&self, answer: &str) -> Option<ExpandedSecretKey> {
        let public_key = byte_array_to_string(self.public_key.as_bytes()).unwrap();
        let combined = format!("{}{}", answer, public_key);
        let mut hasher = Sha512::new();
        hasher.update(combined);
        let hash = hasher.finalize();

        let mut decrypted_seed = [0u8; SECRET_KEY_LENGTH];
        for (i, byte) in self.seed.0.iter().enumerate() {
            decrypted_seed[i] = *byte ^ hash[i & 63];
        }

        ExpandedSecretKey::from_bytes(&decrypted_seed).ok()
    }
}
