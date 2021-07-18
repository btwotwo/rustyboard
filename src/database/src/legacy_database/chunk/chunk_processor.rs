use std::error::Error;

use crate::{legacy_database::index::db_post_ref::ChunkSettings, post::{Post, PostMessage}};

use super::chunk::{
    ChunkError::{self, ChunkTooLarge},
    ChunkTrait,
};
use thiserror::Error;
// TODO: Tests
pub trait ChunkCollectionProcessor {
    type Error: Error;

    fn insert(&mut self, post: &PostMessage) -> Result<ChunkSettings, Self::Error>;
    fn insert_into_existing(
        &mut self,
        chunk: &ChunkSettings,
        post: &PostMessage,
    ) -> Result<(), Self::Error>;
}

pub struct OnDiskChunkCollectionProcessor<TChunk: ChunkTrait> {
    last_chunk: TChunk,
}

#[derive(Debug, Error)]
pub enum OnDiskChunkCollectionProcessorError {
    #[error("Chunk error")]
    ChunkError {
        #[from]
        source: ChunkError,
    },
}

impl<TChunk: ChunkTrait> OnDiskChunkCollectionProcessor<TChunk> {
    pub fn new(max_chunk_size: Option<u64>) -> Result<Self, OnDiskChunkCollectionProcessorError> {
        Ok(OnDiskChunkCollectionProcessor {
            last_chunk: TChunk::try_new(max_chunk_size)?,
        })
    }

    fn extend_current_chunk(&mut self) -> Result<(), OnDiskChunkCollectionProcessorError> {
        let new_chunk = self.last_chunk.create_extended()?;
        self.last_chunk = new_chunk;
        Ok(())
    }
}

impl<TChunk: ChunkTrait> ChunkCollectionProcessor for OnDiskChunkCollectionProcessor<TChunk> {
    type Error = OnDiskChunkCollectionProcessorError;

    fn insert(&mut self, post: &PostMessage) -> Result<ChunkSettings, Self::Error> {
        let post_bytes = post.get_bytes();
        let result = self
            .last_chunk
            .try_append_data(&post_bytes)
            .map(|offset| ChunkSettings {
                chunk_index: self.last_chunk.index(),
                offset,
            });
        match result {
            Err(err) => match err {
                ChunkTooLarge => {
                    self.extend_current_chunk()?;
                    self.insert(post)
                }
                _ => Err(err.into()),
            },
            Ok(settings) => Ok(settings),
        }
    }

    fn insert_into_existing(
        &mut self,
        settings: &ChunkSettings,
        post: &PostMessage,
    ) -> Result<(), Self::Error> {
        let post_bytes = post.get_bytes();
        let mut chunk = TChunk::open(settings.chunk_index)?;
        chunk.try_write_data(&post_bytes, settings.offset)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::legacy_database::chunk::chunk::*;
    use mockall::{automock, mock, predicate::*};

    #[test]
    fn extend_assigns_chunk_to_self_last_chunk() {
        let mut original = mock();
        let mut new_chunk = mock();

        new_chunk.expect_index().return_const(1u64);

        original
            .expect_create_extended()
            .return_once(move || Ok(new_chunk));

        let mut prcsr = OnDiskChunkCollectionProcessor {
            last_chunk: original,
        };

        prcsr.extend_current_chunk().unwrap();
        assert_eq!(prcsr.last_chunk.index(), 1)
    }

    #[test]
    fn insert_when_chunk_too_large_extends_chunk() {
        let mut original = mock();
        let mut new = mock();

        new.expect_try_append_data()
            .withf_st(move |x| x == post().get_bytes())
            .returning(|_| Ok(10));
        new.expect_index().return_const(1u64);

        original
            .expect_try_append_data()
            .withf_st(move |x| x == post().get_bytes())
            .returning(|_| Err(ChunkTooLarge));
        original
            .expect_create_extended()
            .times(1)
            .return_once(move || Ok(new));
        original.expect_index().return_const(0u64);

        let mut prcsr = processor(original);
        prcsr.insert(&post()).unwrap();
    }

    #[test]
    fn insert_should_return_correct_offset_and_index() {
        let mut chunk = mock();
        chunk.expect_try_append_data().returning(|_| Ok(10u64));
        chunk.expect_index().return_const(0u64);

        let mut prcsr = processor(chunk);

        let res = prcsr
            .insert(&post())
            .unwrap();
        assert_eq!(res.chunk_index, 0);
        assert_eq!(res.offset, 10);
    }

    #[test]
    fn insert_into_existsing_should_write_data() {
        let ctx = MockChunkTrait::open_context();
        let offset = 10u64;

        ctx.expect().with(eq(0)).returning(move |_| {
            let mut chunk = mock();
            chunk.expect_index().return_const(0u64);
            chunk
                .expect_try_write_data()
                .withf_st(move |x, off| -> bool {
                    (x == post().get_bytes()) && (off == &offset)
                })
                .returning(|_, _| Ok(()));

            Ok(chunk)
        });
        let chunk = MockChunkTrait::open(0).unwrap();

        let mut prcrsr = processor(chunk);
        prcrsr
            .insert_into_existing(
                &ChunkSettings {
                    chunk_index: 0,
                    offset,
                },
                &post(),
            )
            .unwrap();
    }

    fn post() -> PostMessage {
        PostMessage::new("test".to_string())
    }

    fn mock() -> MockChunkTrait {
        MockChunkTrait::new()
    }

    fn processor(c: MockChunkTrait) -> OnDiskChunkCollectionProcessor<MockChunkTrait> {
        OnDiskChunkCollectionProcessor { last_chunk: c }
    }
}
