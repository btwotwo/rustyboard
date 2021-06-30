use serde::{Deserialize, Serialize};

use super::db_post_ref::{DbPostRef, DbPostRefHash};
/// Reference of post messages, which are stored in chunks. This struct is serialized and written into
/// `index-3.json` to save message positions inside chunks.
#[derive(Serialize, Deserialize)]
pub struct DbPostRefSerialized {
    /// Post hash
    #[serde(rename = "h")]
    hash: String,

    /// Hash of the parent post
    #[serde(rename = "r")]
    reply_to: String,

    /// Offset in bytes from the start of the chunk
    #[serde(rename = "o")]
    offset: u64,

    /// Length of the post message in bytes
    #[serde(rename = "l")]
    length: u64,

    /// Is post deleted. If it's deleted, it won't be shown in list of the posts
    #[serde(rename = "d")]
    deleted: bool,

    /// Chunk filename
    #[serde(rename = "f")]
    chunk_name: String,
}

/// A collection of database references
#[derive(Serialize, Deserialize)]
pub struct IndexCollection {
    pub indexes: Vec<DbPostRefSerialized>,
}

pub struct Hashes {
    pub parent: DbPostRefHash,
    pub hash: DbPostRefHash
}

impl DbPostRefSerialized {
    pub fn split(self) -> (Hashes, DbPostRef) {
        let hash = self.hash;
        let parent = self.reply_to;
        let hashes = Hashes {
            parent, hash
        };

        let db_post_ref = DbPostRef {
            chunk_name: self.chunk_name,
            deleted: self.deleted,
            length: self.length,
            offset: self.offset,
        };

        (hashes, db_post_ref)
    }
}
