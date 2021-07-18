use std::io::{self, BufReader};

use super::{chunk::{ChunkError, chunk_processor::ChunkCollectionProcessor}, index::{serialized::IndexCollection, DbRefCollection}};
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

struct LegacyDatabase<TProcessor: ChunkCollectionProcessor> {
    reference: DbRefCollection,
    chunk_processor: TProcessor
}

impl<TProcessor: ChunkCollectionProcessor> LegacyDatabase<TProcessor> {
    pub fn new(index_file: std::fs::File, chunk_processor: TProcessor) -> LegacyDatabaseResult<Self> {
        let index: IndexCollection = serde_json::from_reader(BufReader::new(index_file))?;
        let reference = DbRefCollection::new(index);

        Ok(LegacyDatabase {
            reference,
            chunk_processor
        })
    }
}

impl<TProcessor: ChunkCollectionProcessor> Database for LegacyDatabase<TProcessor> where LegacyDatabaseError: From<<TProcessor as ChunkCollectionProcessor>::Error>{
    type Error = LegacyDatabaseError;

    fn put_post(&mut self, post: Post, allow_reput: bool) -> Result<(), LegacyDatabaseError> {
        //todo allow_reput
        //todo validate post
        let (hash, message) = self.reference.put_post(post);
        let db_ref = self.reference.get_ref_mut(&hash).unwrap();
        if let Some(settings) = &db_ref.chunk_settings {
            self.chunk_processor.insert_into_existing(&settings,&message)?;
        } else {
            let chunk_settings = self.chunk_processor.insert(&message)?;
            db_ref.chunk_settings = Some(chunk_settings);
        };

        Ok(())
    }

    fn get_post(&self, hash: String) -> Option<Post> {
        todo!()
    }
}
