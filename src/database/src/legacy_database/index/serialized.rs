use std::{fs::File, io::BufReader};

use serde::{Deserialize, Serialize};

use crate::legacy_database::chunk::{chunk_index_to_name, chunk_name_to_index};

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

impl IndexCollection {
    pub fn from_file(file: File) -> serde_json::Result<Self> {
        serde_json::from_reader(BufReader::new(file))
    }
}

impl DbPostRefSerialized {
    pub fn split(self) -> (PostHashes, DbPostRef) {
        let hash = self.hash;
        let parent = self.reply_to;
        let hashes = PostHashes {
            parent: DbPostRefHash::new(parent),
            hash: DbPostRefHash::new(hash),
        };
        let chunk_idx = self.chunk_name.map(chunk_name_to_index);
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
            parent_hash: hashes.parent.clone(),
        };

        (hashes, db_post_ref)
    }

    pub fn new(hashes: &PostHashes, db_ref: &DbPostRef) -> Self {
        let (chunk_name, offset) = match &db_ref.chunk_settings {
            Some(settings) => (
                Some(chunk_index_to_name(settings.chunk_index)),
                settings.offset,
            ),
            None => (None, 0),
        };

        DbPostRefSerialized {
            chunk_name,
            deleted: db_ref.deleted,
            hash: hashes.hash.to_string(),
            reply_to: hashes.parent.to_string(),
            length: db_ref.length,
            offset,
        }
    }

    pub fn serialize(&self) -> serde_json::Result<String> {
        serde_json::to_string(&self)
    }

    pub fn deserialize(source: &str) -> serde_json::Result<Self> {
        serde_json::from_str(source)
    }
}
