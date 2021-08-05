use std::collections::HashMap;

use crate::tests::test_utils::*;
use crate::{
    legacy_database::{
        self,
        chunk::{chunk_processor::ChunkCollectionProcessor, ChunkError},
        index::{
            db_post_ref::{ChunkSettings, DbPostRef},
            diff::Diff,
            serialized::{DbPostRefSerialized, PostHashes},
        },
    },
    post::{Post, PostMessage},
};

pub struct CollectingDiffWithData {
    pub data: Vec<DbPostRefSerialized>,
}

impl Diff for CollectingDiffWithData {
    fn append(
        &self,
        hashes: &PostHashes,
        db_ref: &DbPostRef,
    ) -> legacy_database::index::diff::DiffResult<()> {
        self.data.push(DbPostRefSerialized::new(hashes, db_ref));
        Ok(())
    }

    fn drain() -> legacy_database::index::diff::DiffResult<(Self, Vec<DbPostRefSerialized>)> {
        let ref_1 = some_raw_ref("1", "0", 10);
        let ref_2 = some_raw_ref("2", "1", 5);
        let ref_3 = some_raw_ref("3", "1", 10);

        Ok((
            CollectingDiffWithData { data: Vec::new() },
            vec![ref_1, ref_2, ref_3],
        ))
    }
}

pub struct CollectingChunkProcessor {
    pub data: HashMap<ChunkSettings, PostMessage>,
    pub offset: u64,
}

impl ChunkCollectionProcessor for CollectingChunkProcessor {
    type Error = ChunkError;

    fn insert(&mut self, post: &PostMessage) -> Result<ChunkSettings, Self::Error> {
        let sets = ChunkSettings {
            chunk_index: 0,
            offset: self.offset,
        };
        self.data.insert(sets.clone(), post.clone());
        self.offset += post.get_bytes().len() as u64;

        Ok(sets)
    }

    fn insert_into_existing(
        &mut self,
        chunk: &ChunkSettings,
        post: &PostMessage,
    ) -> Result<(), Self::Error> {
        self.data.insert(chunk.clone(), post.clone());
        Ok(())
    }

    fn get_message(&self, chunk: &ChunkSettings, _len: u64) -> Result<PostMessage, Self::Error> {
        Ok(self.data.get(&chunk).unwrap().clone())
    }

    fn remove(&mut self, chunk: &ChunkSettings) -> Result<(), Self::Error> {
        self.data.remove(chunk);
        Ok(())
    }
}
