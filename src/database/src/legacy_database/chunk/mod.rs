mod chunk;
pub mod chunk_processor;

pub use chunk::ChunkError;
pub use chunk::ChunkIndex;

use self::chunk::Chunk;

pub fn chunk_name_to_index(name: String) -> ChunkIndex {
    Chunk::name_to_index(name)
}

pub fn chunk_index_to_name(index: ChunkIndex) -> String {
    Chunk::index_to_name(index)
}
