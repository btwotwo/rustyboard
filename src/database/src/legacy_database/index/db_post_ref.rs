use std::rc::Rc;

use crate::legacy_database::chunk::ChunkIndex;

/// Post hash, immutable
pub type DbPostRefHash = Rc<String>;

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct ChunkSettings {
    /// Post's chunk index
    pub chunk_index: ChunkIndex,

    /// Offset from the start of the chunk file,
    pub offset: u64,
}

#[derive(Debug, PartialEq)]
pub struct DbPostRef {
    /// Chunk settings. `None` if post was deleted from database and its space was reused,
    /// so the message of the post is not occupying any space in the chunk.
    pub chunk_settings: Option<ChunkSettings>,

    /// Post length in bytes
    pub length: u64,

    /// Is post deleted from the database.
    ///
    /// It still may occupy some place in chunk, see `chunk_settings`.
    pub deleted: bool,

    pub parent_hash: DbPostRefHash,
}
