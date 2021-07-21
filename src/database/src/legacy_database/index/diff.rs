use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
    iter,
};

use super::{
    db_post_ref::DbPostRef,
    serialized::{DbPostRefSerialized, PostHashes},
    DbRefCollection,
};
use thiserror::Error;

const DIFF_FILENAME: &str = "diff-3.list";

#[derive(Debug, Error)]
pub enum DiffFileError {
    #[error("Error serializing reference")]
    SerializationError(#[from] serde_json::Error),
    #[error("Error saving reference")]
    SavingError(#[from] std::io::Error),
}

pub type DiffResult<T> = Result<T, DiffFileError>;
pub trait Diff: Sized {
    fn append(
        &mut self,
        hashes: PostHashes,
        db_ref: DbPostRef,
    ) -> DiffResult<(PostHashes, DbPostRef)>;

    fn apply_diff(&self, coll: &mut DbRefCollection) -> DiffResult<()>;
}

//todo: Open file for every write, or force user to apply the existing diff to the database before opening a new one?
//todo: test error: thread 'legacy_database::index::tests::new::when_contains_deleted_post_should_add_to_deleted' has overflowed its stack

pub struct DiffFile;

impl DiffFile {
    fn deserialize(file: &File) -> Vec<DbPostRefSerialized> {
        let buf = BufReader::new(file);
        buf.lines()
            .map(|l| l.expect("Invalid diff file!"))
            .map(|line| DbPostRefSerialized::deserialize(&line).expect("Invalid diff file!"))
            .collect()
    }
}

impl Diff for DiffFile {
    fn append(
        &mut self,
        hashes: PostHashes,
        db_ref: DbPostRef,
    ) -> DiffResult<(PostHashes, DbPostRef)> {
        let serialized_obj = DbPostRefSerialized::new(hashes, db_ref);
        let serialized_string = serialized_obj.serialize()?;
        //self.0.write_all(format!("{}\n", serialized_string).as_bytes())?;

        Ok(serialized_obj.split())
    }

    fn apply_diff(&self, coll: &mut DbRefCollection) -> DiffResult<()> {
        let diff_file = File::open(DIFF_FILENAME);
        let refs = match diff_file {
            Ok(file) => Self::deserialize(&file),
            Err(io) => match io.kind() {
                std::io::ErrorKind::NotFound => iter::empty().collect(),
                _ => return Err(io.into()),
            },
        };

        //todo Apply parsed refs to collection

        Ok(())
    }
}
