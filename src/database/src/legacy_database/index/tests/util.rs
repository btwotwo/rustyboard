use std::rc::Rc;

use crate::legacy_database::{
    self,
    index::{
        db_post_ref::{ChunkSettings, DbPostRef, DbPostRefHash},
        diff::Diff,
        serialized::{DbPostRefSerialized, IndexCollection, PostHashes},
        DbRefCollection,
    },
};

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
        Ok((DummyDiff, Vec::new()))
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

pub fn collection(refs: Vec<DbPostRefSerialized>) -> DbRefCollection<DummyDiff> {
    DbRefCollection::new(IndexCollection { indexes: refs }).unwrap()
}
