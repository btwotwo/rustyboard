use std::io::{self, BufReader};

use super::{chunk::{Chunk, ChunkError}, index::{Reference, serialized::IndexCollection}};
use crate::{post::Post, post_database::Database};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LegacyDatabaseError {
    #[error("Chunk error")]
    ChunkError {
        #[from]
        source: ChunkError,
    },

    #[error("IO error")]
    IoError {
        #[from]
        source: io::Error
    },

    #[error("Serde error")]
    SerdeError {
        #[from]
        source: serde_json::Error
    }
}

const INDEX_FILENAME: &str = "index-3.json";
const DIFF_FILENAME: &str = "diff-3.list";
pub type LegacyDatabaseResult<T> = Result<T, LegacyDatabaseError>;

struct LegacyDatabase {
    current_chunk: Chunk,
    reference: Reference
}

impl LegacyDatabase {
    pub fn new(index_file: std::fs::File) -> LegacyDatabaseResult<Self> {
        let index: IndexCollection = serde_json::from_reader(BufReader::new(index_file))?;
        let reference= Reference::new(index);

        Ok(LegacyDatabase {
            current_chunk: Chunk::try_new(None)?,
            reference
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
