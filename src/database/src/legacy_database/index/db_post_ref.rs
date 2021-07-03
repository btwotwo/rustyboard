/// Post hash, immutable
pub type DbPostRefHash = String;

#[derive(Debug, PartialEq)]
pub struct DbPostRef {
    /// Offset from the start of the chunk file
    pub offset: u64,

    /// Post length in bytes
    pub length: u64,

    /// Is post deleted from the database
    pub deleted: bool,

    /// Chunk name which contains the post
    pub chunk_name: String,
}

