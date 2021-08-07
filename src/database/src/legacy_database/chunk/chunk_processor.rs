use std::{error::Error, string};

use crate::{legacy_database::index::db_post_ref::ChunkSettings, post::PostMessage};

use super::chunk::{
    ChunkError::{self, ChunkTooLarge},
    ChunkTrait,
};
use thiserror::Error;
pub trait ChunkCollectionProcessor {
    type Error: Error;

    fn insert(&mut self, post: &PostMessage) -> Result<ChunkSettings, Self::Error>;
    fn insert_into_existing(
        &mut self,
        chunk: &ChunkSettings,
        post: &PostMessage,
    ) -> Result<(), Self::Error>;

    fn remove(&mut self, chunk: &ChunkSettings, len: u64) -> Result<(), Self::Error>;

    fn get_message(&self, chunk: &ChunkSettings, len: u64) -> Result<PostMessage, Self::Error>;
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

    #[error("Error converting message bytes to utf8")]
    Base64Error(#[from] string::FromUtf8Error),
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
        let mut chunk = TChunk::open_without_sizecheck(settings.chunk_index)?;
        chunk.try_write_data(&post_bytes, settings.offset)?;
        Ok(())
    }

    fn get_message(
        &self,
        chunk_settings: &ChunkSettings,
        len: u64,
    ) -> Result<PostMessage, Self::Error> {
        let offset = chunk_settings.offset;
        let post_bytes = if self.last_chunk.index() == chunk_settings.chunk_index {
            self.last_chunk.read_data(offset, len)?
        } else {
            TChunk::open_without_sizecheck(chunk_settings.chunk_index)?.read_data(offset, len)?
        };

        let post_message = PostMessage::from_bytes(post_bytes)?;

        Ok(post_message)
    }

    fn remove(&mut self, chunk: &ChunkSettings, len: u64) -> Result<(), Self::Error> {
        TChunk::open_without_sizecheck(chunk.chunk_index)?.remove_data(chunk.offset, len)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::legacy_database::chunk::chunk::*;
    use mockall::predicate::*;

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

        let res = prcsr.insert(&post()).unwrap();
        assert_eq!(res.chunk_index, 0);
        assert_eq!(res.offset, 10);
    }

    #[test]
    fn insert_into_existsing_should_write_data() {
        let ctx = MockChunkTrait::open_without_sizecheck_context();
        let offset = 10u64;

        ctx.expect().with(eq(0)).returning(move |_| {
            let mut chunk = mock();
            chunk.expect_index().return_const(0u64);
            chunk
                .expect_try_write_data()
                .withf_st(move |x, off| -> bool { (x == post().get_bytes()) && (off == &offset) })
                .returning(|_, _| Ok(()));

            Ok(chunk)
        });
        let chunk = MockChunkTrait::open_without_sizecheck(0).unwrap();

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

    #[test]
    fn get_message_selects_current_chunk_if_last_chunk_index_is_same() {
        let mut chunk = mock();
        let offset = 123;
        let len = 321;
        let chunk_settings = ChunkSettings {
            chunk_index: 0,
            offset,
        };

        with_index(&mut chunk, 0);
        chunk
            .expect_read_data()
            .with(eq(chunk_settings.offset), eq(len))
            .times(1)
            .return_once(|_, _| Ok("test".as_bytes().to_vec()));

        let prcsrs = processor(chunk);

        prcsrs.get_message(&chunk_settings, len).unwrap();
    }

    #[test]
    fn get_message_opens_new_chunk_if_chunk_index_is_different() {
        let mut chunk = mock();
        let offset = 123;
        let len = 321;
        let chunk_settings = ChunkSettings {
            chunk_index: 1,
            offset,
        };

        with_index(&mut chunk, 0);
        chunk
            .expect_read_data()
            .with(eq(chunk_settings.offset), eq(len))
            .times(0);
        let ctx = MockChunkTrait::open_without_sizecheck_context();

        ctx.expect().with(eq(1)).returning(move |_| {
            let mut new_chunk = mock();
            new_chunk
                .expect_read_data()
                .with(eq(offset), eq(len))
                .times(1)
                .return_once(|_, _| Ok("test".as_bytes().to_vec()));
            Ok(new_chunk)
        });

        let processor = processor(chunk);
        processor.get_message(&chunk_settings, len).unwrap();
    }

    #[test]
    fn get_message_returns_valid_message() {
        let mut chunk = mock();
        let settings = ChunkSettings {
            chunk_index: 0,
            offset: 123,
        };
        let len = 321;
        with_index(&mut chunk, 0);
        chunk
            .expect_read_data()
            .with(eq(settings.offset), eq(len))
            .times(1)
            .return_once(|_, _| Ok("test".as_bytes().to_vec()));
        let expected_result = PostMessage::new("test".to_string());
        let processor = processor(chunk);

        let result = processor.get_message(&settings, len).unwrap();

        assert_eq!(result, expected_result);
    }

    fn post() -> PostMessage {
        PostMessage::new("test".to_string())
    }

    fn mock() -> MockChunkTrait {
        MockChunkTrait::new()
    }

    fn with_index(mock: &mut MockChunkTrait, index: u64) {
        mock.expect_index().return_const(index);
    }

    fn processor(c: MockChunkTrait) -> OnDiskChunkCollectionProcessor<MockChunkTrait> {
        OnDiskChunkCollectionProcessor { last_chunk: c }
    }
}
