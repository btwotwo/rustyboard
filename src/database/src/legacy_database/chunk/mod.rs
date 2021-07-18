//todo make chunk private, work only through chunk_processor
mod chunk;
pub mod chunk_processor;

pub use chunk::ChunkIndex;
pub use chunk::ChunkError;

use self::chunk::Chunk;

pub fn chunk_name_to_index(name: String) -> ChunkIndex {
    Chunk::name_to_index(name)
}