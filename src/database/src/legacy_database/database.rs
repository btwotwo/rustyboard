use super::chunk::{Chunk, ChunkError};
use crate::{post::Post, post_database::Database};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LegacyDatabaseError {
    #[error("Chunk error")]
    ChunkError {
        #[from]
        source: ChunkError
    }
}

pub type LegacyDatabaseResult<T> = Result<T, LegacyDatabaseError>;

struct LegacyDatabase {
    current_chunk: Chunk,
}

impl LegacyDatabase {
    pub fn new() -> LegacyDatabaseResult<Self> {
        Ok(LegacyDatabase {
            current_chunk: Chunk::try_new(None)?
        })
    }
}

impl Database for LegacyDatabase {
    fn put_post(&mut self, post: Post, allow_reput: bool) {
        todo!()
    }

    fn get_post(&self, hash: String) -> Option<Post> {
        todo!()
    }
}
