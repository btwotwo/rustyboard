use thiserror::Error;

use crate::{
    legacy_database::{
        self,
        chunk::chunk_processor::ChunkCollectionProcessor,
        database::LegacyDatabaseError,
        index::{
            db_post_ref::{ChunkSettings, DbPostRef},
            diff::Diff,
            serialized::DbPostRefSerialized,
        },
    },
    post::*,
};

#[derive(Debug, Error)]
pub enum DummyChunkProcessorError {}
pub struct DummyChunkProcessor;
impl ChunkCollectionProcessor for DummyChunkProcessor {
    type Error = DummyChunkProcessorError;

    fn insert(&mut self, _post: &PostMessage) -> Result<ChunkSettings, Self::Error> {
        Ok(ChunkSettings {
            chunk_index: 0,
            offset: 0,
        })
    }

    fn insert_into_existing(
        &mut self,
        _chunk: &ChunkSettings,
        _post: &PostMessage,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn get_message(&self, _chunk: &ChunkSettings, _len: u64) -> Result<PostMessage, Self::Error> {
        Ok(PostMessage::new("Msg".to_string()))
    }
}
impl From<<DummyChunkProcessor as ChunkCollectionProcessor>::Error> for LegacyDatabaseError {
    fn from(_: <DummyChunkProcessor as ChunkCollectionProcessor>::Error) -> Self {
        LegacyDatabaseError::ChunkError {
            source: legacy_database::chunk::ChunkError::ChunkFileDoesNotExist,
        }
    }
}
pub struct DummyDiff;
impl Diff for DummyDiff {
    fn append(
        &mut self,
        _hashes: &legacy_database::index::serialized::PostHashes,
        _db_ref: &DbPostRef,
    ) -> legacy_database::index::diff::DiffResult<()> {
        Ok(())
    }

    fn drain() -> legacy_database::index::diff::DiffResult<(Self, Vec<DbPostRefSerialized>)> {
        Ok((Self, Vec::new()))
    }
}
