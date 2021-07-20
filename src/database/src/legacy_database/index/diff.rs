use std::{fs::File, io::Write};

use super::{db_post_ref::DbPostRef, serialized::{DbPostRefSerialized, PostHashes}};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DiffFileError {
    #[error("Error serializing reference")]
    SerializationError(#[from] serde_json::Error),
    #[error("Error saving reference")]
    SavingError(#[from] std::io::Error)
}

pub type DiffResult<T> = Result<T, DiffFileError>;
pub trait Diff {
    fn append(&mut self, hashes: PostHashes, db_ref: DbPostRef) -> DiffResult<(PostHashes, DbPostRef)>;   
}

struct DiffFile(File);
impl Diff for DiffFile {
    fn append(&mut self, hashes: PostHashes, db_ref: DbPostRef) -> DiffResult<(PostHashes, DbPostRef)> {
        let serialized_obj = DbPostRefSerialized::new(hashes, db_ref);
        let serialized_string = serialized_obj.serialize()?;
        self.0.write_all(format!("{}\n", serialized_string).as_bytes())?;

        Ok(serialized_obj.split())
    }
}