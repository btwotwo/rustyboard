use std::rc::Rc;

use crate::legacy_database::index::{
    db_post_ref::{ChunkSettings, DbPostRef, DbPostRefHash},
    serialized::DbPostRefSerialized,
};

pub fn rc(hash: &str) -> DbPostRefHash {
    Rc::new(hash.to_string())
}

pub fn some_ref(length: u64) -> DbPostRef {
    DbPostRef {
        chunk_settings: Some(ChunkSettings {
            chunk_index: 0,
            offset: 1,
        }),
        deleted: false,
        length,
    }
}

pub fn some_raw_ref(hash: &str, parent: &str, length: u64) -> DbPostRefSerialized {
    DbPostRefSerialized {
        hash: hash.to_string(),
        reply_to: parent.to_string(),
        offset: 1,
        length,
        deleted: false,
        chunk_name: Some("0.db3".to_string())
    }
}
