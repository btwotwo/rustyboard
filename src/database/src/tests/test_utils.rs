#[macro_export]
macro_rules! in_temp_dir {
    ($block:block) => {
        use std::env::set_current_dir;
        use tempdir::TempDir;
        let tmpdir = TempDir::new("db").unwrap();
        set_current_dir(&tmpdir).unwrap();

        $block;
    };
}

#[macro_export]
macro_rules! assert_ok {
    ($left:expr, $right:expr) => ({
        assert!(matches!($left, Ok($right)))  
    });

    ($left:expr) => ({
        assert!(matches!($left, Ok(())))
    })
}

#[macro_export]
macro_rules! assert_err {
    ($left:expr, $error:path) => ({
        assert!(matches!($left, Err($error)))  
    });
}

use std::rc::Rc;

use crate::{
    legacy_database::{
        self,
        chunk::chunk_processor::ChunkCollectionProcessor,
        index::{
            db_post_ref::{ChunkSettings, DbPostRef, DbPostRefHash},
            diff::Diff,
            serialized::{DbPostRefSerialized, IndexCollection, PostHashes},
            DbRefCollection,
        },
    },
    post::{Post, PostMessage},
};

pub use super::dummy_impls::*;

pub struct CollectingDiffWithData {
    pub data: Vec<DbPostRefSerialized>,
}

impl Diff for CollectingDiffWithData {
    fn append(
        &mut self,
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

pub fn rc(hash: &str) -> DbPostRefHash {
    Rc::new(hash.to_string())
}

pub fn some_ref(length: u64, parent: &str) -> DbPostRef {
    DbPostRef {
        chunk_settings: Some(ChunkSettings {
            chunk_index: 0,
            offset: 1,
        }),
        deleted: false,
        length,
        parent_hash: rc(parent),
    }
}

pub fn some_raw_ref(hash: &str, parent: &str, length: u64) -> DbPostRefSerialized {
    DbPostRefSerialized {
        hash: hash.to_string(),
        reply_to: parent.to_string(),
        offset: 1,
        length,
        deleted: false,
        chunk_name: Some("0.db3".to_string()),
    }
}

/// Ref without reclaimed space
pub fn some_raw_deleted_ref(hash: &str, parent: &str, length: u64) -> DbPostRefSerialized {
    DbPostRefSerialized {
        hash: hash.to_string(),
        reply_to: parent.to_string(),
        offset: 1,
        length,
        deleted: true,
        chunk_name: Some("0.db3".to_string()),
    }
}

/// Ref with reclaimed space
pub fn some_raw_removed_ref(hash: &str, parent: &str) -> DbPostRefSerialized {
    DbPostRefSerialized {
        hash: hash.to_string(),
        reply_to: parent.to_string(),
        offset: 1,
        length: 0,
        deleted: true,
        chunk_name: None,
    }
}

pub fn some_post(hash: &str, parent: &str, message: &str) -> Post {
    Post {
        hash: hash.to_string(),
        message: PostMessage::new(message.to_string()),
        reply_to: parent.to_string(),
    }
}

pub fn collection(refs: Vec<DbPostRefSerialized>) -> DbRefCollection<DummyDiff> {
    DbRefCollection::new(IndexCollection { indexes: refs }).unwrap()
}

pub fn collection_with_diff(
    refs: Vec<DbPostRefSerialized>,
) -> DbRefCollection<CollectingDiffWithData> {
    DbRefCollection::new(IndexCollection { indexes: refs }).unwrap()
}

pub fn dummy_chunk_processor() -> DummyChunkProcessor {
    DummyChunkProcessor
}
