use std::{fs::{self, File, OpenOptions}, io::{self, BufRead, BufReader, Write}, iter::{self, FromIterator}, mem, path::{self, Path}};

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
        hashes: &PostHashes,
        db_ref: &DbPostRef,
    ) -> DiffResult<()>;

    fn drain(&mut self) -> DiffResult<Vec<DbPostRefSerialized>>;
}

pub struct DiffFile(File);

impl DiffFile {
    pub fn new() -> DiffResult<Self> {
        let file = Self::create_file()?;
        Ok(DiffFile(file))
    }

    fn create_file() -> io::Result<File> {
        let file_path = Path::new(DIFF_FILENAME);
        if !file_path.exists() {
            File::create(&file_path)?;
        }
        let file = OpenOptions::new().append(true).create(true).read(true).open(&file_path)?;
        Ok(file)
    }
}

impl Diff for DiffFile {
    fn append(
        &mut self,
        hashes: &PostHashes,
        db_ref: &DbPostRef,
    ) -> DiffResult<()> {
        let serialized_obj = DbPostRefSerialized::new(hashes, db_ref);
        let serialized_string = serialized_obj.serialize()?;
        let mut file = Self::create_file()?;
        file.write_all(format!("{}\n", serialized_string).as_bytes())?;

        Ok(())
    }

    fn drain(&mut self) -> DiffResult<Vec<DbPostRefSerialized>>{
        let diff_file = Self::create_file()?;
        let buf = BufReader::new(&diff_file);

        let result = buf.lines()
            .map(|l| l.expect("Invalid diff file!"))
            .map(|line| DbPostRefSerialized::deserialize(&line).expect("Invalid diff file!"))
            .collect();
        mem::drop(diff_file);
        fs::remove_file(DIFF_FILENAME)?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {

}