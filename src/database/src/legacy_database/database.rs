use std::io;

use super::{
    chunk::{chunk_processor::ChunkCollectionProcessor, ChunkError},
    index::{
        diff::{Diff, DiffFileError},
        DbRefCollection, DbRefCollectionError,
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

    #[error("Can't update non-deleted post.")]
    CantUpdateNonDeletedPost,

    #[error("Error processing DbReferenceCollection")]
    DbRefCollectionError(#[from] DbRefCollectionError),
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
        let (hash, message) = self.reference.put_post(post)?;
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

    fn update_post(&mut self, post: Post) -> Result<(), Self::Error> {
        if !self.reference.ref_exists(&post.hash) {
            return Err(LegacyDatabaseError::PostDoesntExist);
        }

        if !self.reference.ref_deleted(&post.hash) {
            return Err(LegacyDatabaseError::CantUpdateNonDeletedPost);
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

    fn delete_post(&mut self, hash: String) -> Result<(), Self::Error> {
        self.reference.delete_post(&hash)?;
        let chunk_settings = match self.reference.get_ref(&hash).unwrap().chunk_settings {
            None => return Ok(()),
            Some(c) => c,
        };

        self.chunk_processor.remove(&chunk_settings)?;

        todo!("Add tests and delete functionality for chunk processor, for reference collection, for database.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        assert_err, legacy_database::index::db_post_ref::ChunkSettings, post::PostMessage,
        tests::test_utils::*,
    };

    #[test]
    fn update_post_if_post_doesnt_exist_should_return_error() {
        let collection = collection(vec![some_raw_ref("1", "0", 10), some_raw_ref("2", "0", 15)]);
        let mut db = LegacyDatabase::new(collection, dummy_chunk_processor());
        let post = some_post("10", "0", "test");

        let result = db.update_post(post);
        assert_err!(result, LegacyDatabaseError::PostDoesntExist)
    }

    #[test]
    fn update_post_if_post_isnt_deleted_should_return_error() {
        let collection = collection(vec![some_raw_ref("1", "0", 10)]);
        let mut db = LegacyDatabase::new(collection, dummy_chunk_processor());
        let post = some_post("1", "0", "test2");

        let result = db.update_post(post);
        assert_err!(result, LegacyDatabaseError::CantUpdateNonDeletedPost)
    }

    #[test]
    fn put_post_when_post_exists_should_return_error() {
        let collection = collection(vec![some_raw_ref("1", "0", 10)]);
        let mut db = LegacyDatabase::new(collection, dummy_chunk_processor());
        let post = some_post("1", "0", "test");

        let result = db.put_post(post);
        assert_err!(result, LegacyDatabaseError::DuplicatePost)
    }

    #[test]
    fn upsert_post_should_put_into_db_ref_collection() {
        let processor = dummy_chunk_processor();
        let collection = collection(vec![some_raw_ref("1", "0", 10)]);
        let post = some_post("5", "10", "test");
        let mut db = LegacyDatabase::new(collection, processor);

        db.upsert_post(post).unwrap();

        let db_ref = db.reference.get_ref("5").unwrap();
        assert_eq!(db_ref.parent_hash, rc("10"));
        assert_eq!(db_ref.length, 4);
    }

    #[test]
    fn upsert_post_if_no_chunk_set_should_put_into_chunk_and_update_db_ref() {
        let processor = collecting_chunk_processor();
        let collection = collection(vec![]);
        let post = some_post("5", "0", "test");
        let mut db = LegacyDatabase::new(collection, processor);

        db.upsert_post(post).unwrap();

        let expected_chunk_settings = ChunkSettings {
            chunk_index: 0,
            offset: 0,
        };
        let collected = db
            .chunk_processor
            .data
            .get(&expected_chunk_settings)
            .unwrap();
        let db_ref = db.reference.get_ref("5").unwrap();

        assert_eq!(collected, &PostMessage::new("test".to_string()));
        assert_eq!(
            db_ref.chunk_settings.as_ref().unwrap(),
            &expected_chunk_settings
        );
    }

    #[test]
    fn upsert_post_when_chunk_is_set_should_write_into_existing_chunk() {
        let processor = collecting_chunk_processor();
        let mut deleted_ref = some_raw_deleted_ref("1", "0", 9999);
        deleted_ref.chunk_name = Some("10.db3".to_string());
        let collection = collection(vec![deleted_ref]);
        let post = some_post("1", "0", "test");

        let mut db = LegacyDatabase::new(collection, processor);
        db.upsert_post(post).unwrap();

        let expected_chunk_settings = ChunkSettings {
            chunk_index: 10,
            offset: 1,
        };
        let expected_post = PostMessage::new("test".to_string());
        let collected = db
            .chunk_processor
            .data
            .get(&expected_chunk_settings)
            .unwrap();

        assert_eq!(collected, &expected_post);
    }
}
