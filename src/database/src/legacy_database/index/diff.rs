use std::{fs::{File, OpenOptions}, io::Write};

use super::{db_post_ref::DbPostRef, serialized::{DbPostRefSerialized, PostHashes}};
use thiserror::Error;

const DIFF_FILENAME: &str = "diff-3.list";

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
    fn new() -> Self where Self: Sized;
}

//todo: Open file for every write, or force user to apply the existing diff to the database before opening a new one?
//todo: test error: thread 'legacy_database::index::tests::new::when_contains_deleted_post_should_add_to_deleted' has overflowed its stack

pub struct DiffFile;
impl Diff for DiffFile {
    fn append(&mut self, hashes: PostHashes, db_ref: DbPostRef) -> DiffResult<(PostHashes, DbPostRef)> {
        let serialized_obj = DbPostRefSerialized::new(hashes, db_ref);
        let serialized_string = serialized_obj.serialize()?;
        //self.0.write_all(format!("{}\n", serialized_string).as_bytes())?;

        Ok(serialized_obj.split())
    }

    fn new() -> Self where Self: Sized {
        DiffFile{}
    }
}