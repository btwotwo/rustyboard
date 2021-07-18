#[derive(Clone)]
pub struct Post {
    pub hash: String,
    pub reply_to: String,
    /// Base64 encoded post message
    pub message: String,
}

impl Post {
    pub fn new(hash: String, reply_to: String, raw_message: String) -> Self {
        Self {
            hash,
            reply_to,
            message: base64::encode(raw_message),
        }
    }

    pub fn get_message_bytes(&self) -> Vec<u8> {
        base64::decode(&self.message).unwrap() //todo replace with error handling;
    }
}
