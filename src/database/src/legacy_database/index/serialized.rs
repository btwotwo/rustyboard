use serde::{Deserialize, Serialize};

use crate::legacy_database::chunk::chunk::Chunk;

use super::db_post_ref::{ChunkSettings, DbPostRef, DbPostRefHash};
/// Reference of post messages, which are stored in chunks. This struct is serialized and written into
/// `index-3.json` to save message positions inside chunks.
#[derive(Serialize, Deserialize)]
pub struct DbPostRefSerialized {
    /// Post hash
    #[serde(rename = "h")]
    pub hash: String,

    /// Hash of the parent post
    #[serde(rename = "r")]
    pub reply_to: String,

    /// Offset in bytes from the start of the chunk
    #[serde(rename = "o")]
    pub offset: u64,

    /// Length of the post message in bytes
    #[serde(rename = "l")]
    pub length: u64,

    /// Is post deleted. If it's deleted, it won't be shown in list of the posts
    #[serde(rename = "d")]
    pub deleted: bool,

    /// Chunk filename
    #[serde(rename = "f")]
    pub chunk_name: Option<String>,
}

/// A collection of raw deserialized database references
#[derive(Serialize, Deserialize)]
pub struct IndexCollection {
    pub indexes: Vec<DbPostRefSerialized>,
}

pub struct PostHashes {
    pub parent: DbPostRefHash,
    pub hash: DbPostRefHash,
}

impl DbPostRefSerialized {
    pub fn split(self) -> (PostHashes, DbPostRef) {
        let hash = self.hash;
        let parent = self.reply_to;
        let hashes = PostHashes {
            parent: DbPostRefHash::new(parent),
            hash: DbPostRefHash::new(hash),
        };
        let chunk_idx = self.chunk_name.map(Chunk::name_to_index);
        let chunk_settings = match chunk_idx {
            Some(chunk_index) => Some(ChunkSettings {
                chunk_index,
                offset: self.offset,
            }),
            None => None,
        };

        let db_post_ref = DbPostRef {
            chunk_settings,
            deleted: self.deleted,
            length: self.length,
        };

        (hashes, db_post_ref)
    }
}
