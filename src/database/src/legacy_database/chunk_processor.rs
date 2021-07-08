use std::error::Error;

use crate::post::Post;

use super::{index::db_post_ref::ChunkSettings, chunk::Chunk};
use super::chunk::ChunkError::{self, ChunkTooLarge};
use thiserror::Error;
// TODO: Tests
pub trait ChunkCollectionProcessor {
    type Error: Error;

    fn insert(&mut self, post: &Post) -> Result<ChunkSettings, Self::Error>;
    fn insert_into_existing(&mut self, chunk: &ChunkSettings, post: &Post) -> Result<(), Self::Error>;
}

pub struct OnDiskChunkCollectionProcessor {
    last_chunk: Chunk,
}

#[derive(Debug, Error)]
pub enum OnDiskChunkCollectionProcessorError {
    #[error("Chunk error")]
    ChunkError {
        #[from]
        source: ChunkError
    }
}

impl OnDiskChunkCollectionProcessor {
    fn extend_current_chunk(&mut self) -> Result<(), OnDiskChunkCollectionProcessorError> {
        let new_chunk = self.last_chunk.create_extended()?;
        self.last_chunk = new_chunk;
        Ok(())
    }
}

impl ChunkCollectionProcessor for OnDiskChunkCollectionProcessor {
    type Error = OnDiskChunkCollectionProcessorError;

    fn insert(&mut self, post: &Post) -> Result<ChunkSettings, Self::Error> {
        let post_bytes = post.get_bytes();
        let result = self.last_chunk.try_append_data(&post_bytes);
        match result {
            Ok(offset) => Ok(ChunkSettings {
                chunk_index: self.last_chunk.index,
                offset
            }),
            Err(err) => match err {
                ChunkTooLarge => {
                    self.extend_current_chunk()?;
                    self.insert(post)
                },
                _ => Err(err.into())
            }
        }
    }

    fn insert_into_existing(&mut self, settings: &ChunkSettings, post: &Post) -> Result<(), Self::Error> {
        let post_bytes = post.get_bytes();
        let mut chunk = Chunk::open(settings.chunk_index)?;
        chunk.try_write_data(&post_bytes, settings.offset)?;
        Ok(())
    }

}