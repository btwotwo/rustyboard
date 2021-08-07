use database::{legacy_database::{chunk::chunk_processor::{ChunkCollectionProcessor, OnDiskChunkCollectionProcessor}, database::LegacyDatabase, index::{DbRefCollection, diff::DiffFile, serialized::IndexCollection}}, post_database::Database};

pub fn create_database_file() {
    let index_coll = IndexCollection {
        indexes: vec![]
    };
    let ref_collection = DbRefCollection::<DiffFile>::new(index_coll).unwrap();
    let db = LegacyDatabase::new(ref_collection, OnDiskChunkCollectionProcessor::new(None).unwrap());
}

pub fn read_database_file() -> impl Database {
    todo!()
}