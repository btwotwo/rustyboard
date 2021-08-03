use std::fs::{self, File, OpenOptions};
use std::{
    io::{self, BufRead, BufReader, Write},
    mem,
    path::Path,
};

use super::{
    db_post_ref::DbPostRef,
    serialized::{DbPostRefSerialized, PostHashes},
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
    fn append(&self, hashes: &PostHashes, db_ref: &DbPostRef) -> DiffResult<()>;

    fn drain() -> DiffResult<(Self, Vec<DbPostRefSerialized>)>;
}

pub struct DiffFile(File);

impl DiffFile {
    fn new() -> DiffResult<Self> {
        let file = Self::create_file()?;
        Ok(DiffFile(file))
    }

    fn create_file() -> io::Result<File> {
        let file_path = Path::new(DIFF_FILENAME);
        if !file_path.exists() {
            File::create(&file_path)?;
        }
        let file = OpenOptions::new()
            .append(true)
            .create(true)
            .read(true)
            .open(&file_path)?;
        Ok(file)
    }
}

impl Diff for DiffFile {
    fn append(&self, hashes: &PostHashes, db_ref: &DbPostRef) -> DiffResult<()> {
        let serialized_obj = DbPostRefSerialized::new(hashes, db_ref);
        let serialized_string = serialized_obj.serialize()?;
        let mut file = Self::create_file()?;
        file.write_all(format!("{}\n", serialized_string).as_bytes())?;

        Ok(())
    }

    fn drain() -> DiffResult<(Self, Vec<DbPostRefSerialized>)> {
        let diff_file = Self::create_file()?;
        let buf = BufReader::new(&diff_file);

        let result = buf
            .lines()
            .map(|l| l.expect("Invalid diff file!"))
            .map(|line| DbPostRefSerialized::deserialize(&line).expect("Invalid diff file!"))
            .collect();
        mem::drop(diff_file);
        fs::remove_file(DIFF_FILENAME)?;

        Ok((Self::new()?, result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusty_fork::rusty_fork_test;
    use std::{
        fs::{read_to_string, File},
        str,
    };

    use crate::in_temp_dir;

    const SERIALIZED_POSTS: &str = r#"{"h":"1","r":"0","o":5,"l":15,"d":false,"f":"0.db3"}
{"h":"2","r":"1","o":60,"l":133,"d":true,"f":"1.db3"}"#;

    rusty_fork_test! {
        #[test]
        fn drain_should_return_correct_collection() {
            in_temp_dir!({
                create_file();
                let (_, coll) = DiffFile::drain().unwrap();

                assert_eq!(coll[0], ref_1());
                assert_eq!(coll[1], ref_2())
            });
        }
    }

    rusty_fork_test! {
        #[test]
        fn drain_should_empty_file() {
            in_temp_dir!({
                create_file();
                DiffFile::drain().unwrap();

                assert_eq!(read_to_string(DIFF_FILENAME).unwrap(), "".to_string());
            });
        }
    }

    rusty_fork_test! {
        #[test]
        fn append_should_append_correctly() {
            in_temp_dir!({
                let (hashes, post) = ref_1().split();
                let ref_1_ser = format!("{}\n", SERIALIZED_POSTS.split('\n').next().unwrap());
                let (mut diff, _) = DiffFile::drain().unwrap();
                diff.append(&hashes, &post).unwrap();

                assert_eq!(read_to_string(DIFF_FILENAME).unwrap(), ref_1_ser);
            });
        }
    }

    fn create_file() -> File {
        let mut file = DiffFile::create_file().unwrap();
        file.write_all(SERIALIZED_POSTS.as_bytes()).unwrap();

        file
    }

    /// First ref in the SERIALIZED_POSTS
    fn ref_1() -> DbPostRefSerialized {
        DbPostRefSerialized {
            chunk_name: Some("0.db3".to_string()),
            hash: "1".to_string(),
            reply_to: "0".to_string(),
            deleted: false,
            offset: 5,
            length: 15,
        }
    }

    /// Second ref in the SERIALIZED_POSTS
    fn ref_2() -> DbPostRefSerialized {
        DbPostRefSerialized {
            chunk_name: Some("1.db3".to_string()),
            deleted: true,
            hash: "2".to_string(),
            reply_to: "1".to_string(),
            offset: 60,
            length: 133,
        }
    }
}
