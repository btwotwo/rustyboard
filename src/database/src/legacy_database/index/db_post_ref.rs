use crate::legacy_database::chunk::ChunkIndex;

/// Post hash, immutable
pub type DbPostRefHash = String;

#[derive(Debug, PartialEq)]
pub struct DbPostRef {
    /// Offset from the start of the chunk file,
    /// `None` if chunk is not yet defined, or the post is deleted **and** its contents were replaced by another post's message
    pub offset: Option<u64>,

    /// Post length in bytes
    pub length: u64,

    /// Is post deleted from the database
    pub deleted: bool,

    /// Chunk index which has the post message.
    // `None` if chunk is not yet defined, or the post is deleted **and** its contents were replaced by another post's message.
    pub chunk_index: Option<ChunkIndex>,
}
