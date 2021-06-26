use crate::{post::Post, post_database::Database};

use super::chunk::Chunk;

struct NanoboardDatabase {
    current_chunk: Chunk,
}

impl NanoboardDatabase {
    pub fn new() -> Self {
        todo!()
    }
}

impl Database for NanoboardDatabase {
    fn put_post(&mut self, post: Post, allow_reput: bool) {
        todo!()
    }

    fn get_post(&self, hash: String) -> Option<Post> {
        todo!()
    }
}
