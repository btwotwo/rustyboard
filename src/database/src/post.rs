use std::string::FromUtf8Error;

#[derive(Clone)]
pub struct Post {
    pub hash: String,
    pub reply_to: String,
    /// Base64 encoded post message
    pub message: PostMessage,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct PostMessage(String);

impl Post {
    pub fn new(hash: String, reply_to: String, raw_message: String) -> Self {
        Self {
            hash,
            reply_to,
            message: PostMessage::new(raw_message),
        }
    }

    pub fn get_message_bytes(&self) -> Vec<u8> {
        self.message.get_bytes()
    }
}

impl PostMessage {
    pub fn new(raw_message: String) -> Self {
        PostMessage(base64::encode(raw_message))
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, FromUtf8Error> {
        let utf8 = String::from_utf8(bytes)?;
        Ok(Self::new(utf8))
    }

    pub fn get_bytes(&self) -> Vec<u8> {
        base64::decode(&self.0).unwrap()
    }
}
