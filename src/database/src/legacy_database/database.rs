use std::{io, rc::Rc};

use super::{chunk::{chunk_processor::ChunkCollectionProcessor, ChunkError}, index::{DbRefCollection, db_post_ref::DbPostRef, diff::{Diff, DiffFileError}, serialized::IndexCollection}};
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

    #[error("Entry isn't deleted, but chunk settings are not specified. Entry hash: {0}")]
    EntryCorrupted(String)
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

    fn get_post(&self, hash: String) -> Result<Option<Post>, LegacyDatabaseError> {
        let db_ref = match self.reference.get_ref(&hash) {
            Some(db_ref) => db_ref,
            None => return Ok(None)
        };

        if db_ref.deleted {
            return Ok(Some(Post::new(
                hash,
                db_ref.parent_hash.to_string(),
                "Deleted Message Stub Move Me To Const Pls :)".to_string(),
            )));
        }

        let chunk_settings = db_ref.chunk_settings.as_ref().ok_or_else(|| LegacyDatabaseError::EntryCorrupted(hash.clone()))?;
        let post_message = self.chunk_processor.get_message(&chunk_settings, db_ref.length)?;
        Ok(Some(Post {
            hash,
            message: post_message,
            reply_to: db_ref.parent_hash.to_string()
        }))
    }

    fn update_post(&mut self, post: Post) -> Result<(), Self::Error> {
        if !self.reference.ref_exists(&post.hash) {
            return Err(LegacyDatabaseError::PostDoesntExist);
        }

        self.upsert_post(post)?;

        Ok(())
    }
}
