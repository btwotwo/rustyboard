use std::{io, rc::Rc};

use super::{
    chunk::{chunk_processor::ChunkCollectionProcessor, ChunkError},
    index::{
        diff::{Diff, DiffFileError},
        serialized::IndexCollection,
        DbRefCollection,
    },
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

    #[error("Error processing diff")]
    DiffError(#[from] DiffFileError),

    #[error("Trying to add duplicate post!")]
    DuplicatePost,

    #[error("Post does not exist")]
    PostDoesntExist,
}

pub type LegacyDatabaseResult<T> = Result<T, LegacyDatabaseError>;

struct LegacyDatabase<TProcessor, TDiff>
where
    TProcessor: ChunkCollectionProcessor,
    TDiff: Diff,
{
    reference: DbRefCollection<TDiff>,
    chunk_processor: TProcessor,
}

impl<TProcessor: ChunkCollectionProcessor, TDiff: Diff> LegacyDatabase<TProcessor, TDiff>
where
    LegacyDatabaseError: From<<TProcessor as ChunkCollectionProcessor>::Error>,
{
    pub fn new(
        index_file: std::fs::File,
        chunk_processor: TProcessor,
    ) -> LegacyDatabaseResult<Self> {
        let index: IndexCollection = IndexCollection::from_file(index_file)?;
        let reference = DbRefCollection::<TDiff>::new(index)?;

        Ok(LegacyDatabase {
            reference,
            chunk_processor,
        })
    }

    fn upsert_post(&mut self, post: Post) -> Result<(), LegacyDatabaseError> {
        //todo validate post
        let (hash, message) = self.reference.put_post(post);
        let db_ref = self.reference.get_ref_mut(&hash).unwrap();
        match &db_ref.chunk_settings {
            Some(settings) => {
                self.chunk_processor
                    .insert_into_existing(&settings, &message)?;
            }
            None => {
                let chunk_settings = self.chunk_processor.insert(&message)?;
                db_ref.chunk_settings = Some(chunk_settings);
            }
        };
        Ok(())
    }
}

impl<TProcessor: ChunkCollectionProcessor, TDiff: Diff> Database
    for LegacyDatabase<TProcessor, TDiff>
where
    LegacyDatabaseError: From<<TProcessor as ChunkCollectionProcessor>::Error>,
{
    type Error = LegacyDatabaseError;

    fn put_post(&mut self, post: Post) -> Result<(), LegacyDatabaseError> {
        //todo allow_reput
        if self.reference.ref_exists(&post.hash) {
            return Err(LegacyDatabaseError::DuplicatePost);
        }

        self.upsert_post(post)?;

        Ok(())
    }

    fn get_post(&self, hash: String) -> Option<Post> {
        let db_ref = self.reference.get_ref(&hash)?;
        if db_ref.deleted {
            return Some(Post::new(
                hash,
                db_ref.parent_hash.to_string(),
                "Deleted Message Stub Move Me To Const Pls :)".to_string(),
            ));
        }

        let chunk_settings = db_ref.chunk_settings.as_ref()?;


        //todo finish this method
        todo!()
    }

    fn update_post(&mut self, post: Post) -> Result<(), Self::Error> {
        if !self.reference.ref_exists(&post.hash) {
            return Err(LegacyDatabaseError::PostDoesntExist);
        }

        self.upsert_post(post)?;

        Ok(())
    }
}
