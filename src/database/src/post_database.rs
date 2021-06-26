use crate::post::Post;

trait Database {
    fn put_post(&mut self, post: Post, allow_reput: bool);
    fn get_post(&self, hash: String) -> Option<Post>;
}