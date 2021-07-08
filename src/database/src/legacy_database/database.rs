use std::io::{self, BufReader};

use super::{
    chunk::{Chunk, ChunkError, ChunkIndex},
    index::{serialized::IndexCollection, DbRefCollection},
};
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
        source: io::Error,
    },

    #[error("Serde error")]
    SerdeError {
        #[from]
        source: serde_json::Error,
    },
}

const INDEX_FILENAME: &str = "index-3.json";
const DIFF_FILENAME: &str = "diff-3.list";
pub type LegacyDatabaseResult<T> = Result<T, LegacyDatabaseError>;

struct LegacyDatabase {
    reference: DbRefCollection,
    last_chunk: Chunk,
}

impl LegacyDatabase {
    pub fn new(index_file: std::fs::File) -> LegacyDatabaseResult<Self> {
        let index: IndexCollection = serde_json::from_reader(BufReader::new(index_file))?;
        let reference = DbRefCollection::new(index);

        Ok(LegacyDatabase {
            reference,
            last_chunk: Chunk::try_new(None)?,
        })
    }
}

impl Database for LegacyDatabase {
    type Error = LegacyDatabaseError;

    fn put_post(&mut self, post: Post, allow_reput: bool) -> Result<(), LegacyDatabaseError> {
        //todo allow_reput
        //todo validate post
        let post_hash = self.reference.put_post(post);

        // let chunk = match db_post_ref.chunk_index {
        //     Some(idx) => Chunk::open(idx).unwrap(),
        //     None => self.last_chunk,
        // };

        todo!()
    }

    fn get_post(&self, hash: String) -> Option<Post> {
        todo!()
    }
}
