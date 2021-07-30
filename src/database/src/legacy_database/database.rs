use std::io;

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

    #[error("Entry isn't deleted, but chunk settings are not specified. Entry hash: {0}")]
    EntryCorrupted(String),
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

impl<TProcessor, TDiff> LegacyDatabase<TProcessor, TDiff>
where
    LegacyDatabaseError: From<<TProcessor as ChunkCollectionProcessor>::Error>,
    TProcessor: ChunkCollectionProcessor,
    TDiff: Diff,
{
    pub fn new(reference: DbRefCollection<TDiff>, chunk_processor: TProcessor) -> Self {
        LegacyDatabase {
            reference,
            chunk_processor,
        }
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
        if self.reference.ref_exists(&post.hash) {
            return Err(LegacyDatabaseError::DuplicatePost);
        }

        self.upsert_post(post)?;

        Ok(())
    }

    fn get_post(&self, hash: String) -> Result<Option<Post>, LegacyDatabaseError> {
        let db_ref = match self.reference.get_ref(&hash) {
            Some(db_ref) => db_ref,
            None => return Ok(None),
        };

        if db_ref.deleted {
            return Ok(Some(Post::new(
                hash,
                db_ref.parent_hash.to_string(),
                "Deleted Message Stub Move Me To Const Pls :)".to_string(),
            )));
        }

        let chunk_settings = db_ref
            .chunk_settings
            .as_ref()
            .ok_or_else(|| LegacyDatabaseError::EntryCorrupted(hash.clone()))?;

        let post_message = self
            .chunk_processor
            .get_message(&chunk_settings, db_ref.length)?;

        Ok(Some(Post {
            hash,
            message: post_message,
            reply_to: db_ref.parent_hash.to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{assert_err, tests::test_utils::*};

    #[test]
    fn update_post_if_post_doesnt_exist_should_return_error() {
        let collection = collection(vec![some_raw_ref("1", "0", 10), some_raw_ref("2", "0", 15)]);
        let mut db = LegacyDatabase::new(collection, dummy_chunk_processor());
        let post = some_post("10", "0", "test");

        let result = db.update_post(post);
        assert_err!(result, LegacyDatabaseError::PostDoesntExist)
    }

    #[test]
    fn put_post_when_post_exists_should_return_error() {
        let collection = collection(vec![some_raw_ref("1", "0", 10)]);
        let mut db = LegacyDatabase::new(collection, dummy_chunk_processor());
        let post = some_post("1", "0", "test");

        let result = db.put_post(post);
        assert_err!(result, LegacyDatabaseError::DuplicatePost)
    }

    //todo: upsert post + collecting chunk processor
}
