pub struct Post {
    pub hash: String,
    pub reply_to: String,
    /// Base64 encoded post message
    pub message: String,
}

impl Post {
    pub fn get_bytes(&self) -> Vec<u8> {
        base64::decode(&self.message).unwrap() //todo replace with error handling;
    }
}
