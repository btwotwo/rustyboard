use crate::post::Post;
use std::error::Error;

pub trait Database {
    type Error: Error;

    fn put_post(&mut self, post: Post) -> Result<(), Self::Error>;
    fn update_post(&mut self, post: Post) -> Result<(), Self::Error>;
    fn get_post(&self, hash: String) -> Result<Option<Post>, Self::Error>;
    fn delete_post(&mut self, hash: String) -> Result<(), Self::Error>;
}
