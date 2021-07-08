use crate::post::Post;
use std::error::Error;

pub trait Database {
    type Error: Error;

    fn put_post(&mut self, post: Post, allow_reput: bool) -> Result<(), Self::Error>;
    fn get_post(&self, hash: String) -> Option<Post>;
}
