use std::error::Error;
use crate::post::Post;

pub trait Database<TError: Error> {
    fn put_post(&mut self, post: Post, allow_reput: bool) -> Result<(), TError>;
    fn get_post(&self, hash: String) -> Option<Post>;
}
