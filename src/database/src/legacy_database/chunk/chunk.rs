#[cfg(test)]
use mockall::automock;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::BufReader;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::os::unix::prelude::FileExt;
use std::path::Path;
use thiserror::Error;

pub const CHUNK_EXT: &str = ".db3";

#[allow(clippy::identity_op)] // For better readability
const MAX_CHUNK_SIZE: u64 = 1 * 1024 * 1024 * 1024; // 1 GB

pub type ChunkIndex = u64;
pub type Offset = u64;

#[cfg_attr(test, automock)]
pub trait ChunkTrait {
    fn create_extended(&self) -> ChunkResult<Self>
    where
        Self: Sized;

    fn try_append_data(&mut self, data: &[u8]) -> ChunkResult<Offset>;
    fn try_write_data(&mut self, data: &[u8], offset: Offset) -> ChunkResult<()>;
    fn remove_data(&mut self, offset: Offset, length: u64) -> ChunkResult<()>;
    fn open_without_sizecheck(index: ChunkIndex) -> ChunkResult<Self>
    where
        Self: Sized;

    fn try_new(max_chunk_size: Option<u64>) -> ChunkResult<Self>
    where
        Self: Sized;

    fn index(&self) -> ChunkIndex;

    fn read_data(&self, offset: Offset, length: u64) -> ChunkResult<Vec<u8>>;
}
#[derive(Debug)]
pub struct Chunk {
    pub index: ChunkIndex,
    max_chunk_size: u64,
    filename: String,
}
#[derive(Debug, Error)]
pub enum ChunkError {
    #[error("Chunk is exceeding its maximum size")]
    ChunkTooLarge,

    #[error("IO error")]
    IoError {
        #[from]
        source: std::io::Error,
    },

    #[error("Chunk file does not exist")]
    ChunkFileDoesNotExist,
}
pub type ChunkResult<T> = std::result::Result<T, ChunkError>;

enum FileMode {
    Write,
    Append,
    Read,
}

impl ChunkTrait for Chunk {
    /// Creates a new chunk with incremented index
    fn create_extended(&self) -> ChunkResult<Self> {
        let new_index = self.index + 1;
        let chunk = Self::try_create(new_index, Some(self.max_chunk_size))?;

        Ok(chunk)
    }

    /// Appends data to the chunk.
    /// # Errors
    /// If the chunk is too large, will return an error.
    /// # Returns
    /// An offset of the data from the start of file
    fn try_append_data(&mut self, data: &[u8]) -> ChunkResult<Offset> {
        self.validate_chunk_size()?;
        let mut file = self.get_file(FileMode::Append)?;
        let pos = file.seek(SeekFrom::End(0))?;
        file.write_all(data)?;

        Ok(pos)
    }

    /// Writes data into chunk from given offset, does not append, does not validate size
    fn try_write_data(&mut self, data: &[u8], offset: Offset) -> ChunkResult<()> {
        let file = self.get_file(FileMode::Write)?;
        file.write_all_at(data, offset)?;
        Ok(())
    }

    /// Replaces bytes in given offset with zeros
    fn remove_data(&mut self, offset: Offset, length: u64) -> ChunkResult<()> {
        let zeros = vec![0; length as usize];
        self.try_write_data(&zeros, offset)?;

        Ok(())
    }

    /// Tries to open existing chunk with specified index.
    ///
    /// **Warning!** This function doesn't check for chunk's size
    /// # Arguments
    /// * `index` - chunk's index ('0.db3', '1.db3'...)
    fn open_without_sizecheck(index: ChunkIndex) -> ChunkResult<Self> {
        let chunk = Chunk::new(index, None);
        chunk.file_exists()?;
        Ok(chunk)
    }

    /// Tries to open already existing chunk, starting from `0.db3`. If chunk is larger than the limit, tries to open the next one.
    /// # Errors
    /// If any IO error (except [`NotFound`]) is encountered the function will return immediately
    fn try_new(max_chunk_size: Option<u64>) -> ChunkResult<Self> {
        Self::try_new_from(0, max_chunk_size)
    }

    /// Returns chunk index (0 - for "0.db3", 1 - for "1.db3", etc...)
    fn index(&self) -> ChunkIndex {
        self.index
    }

    /// Reads byte array specified via offset from the start of the file and length
    fn read_data(&self, offset: Offset, length: u64) -> ChunkResult<Vec<u8>> {
        let file = self.get_file(FileMode::Read)?;
        let mut buffer = vec![0; length as usize];
        let mut reader = BufReader::new(file);
        reader.seek(SeekFrom::Start(offset))?;
        reader.read_exact(&mut buffer)?;

        Ok(buffer)
    }
}

impl Chunk {
    fn new(index: ChunkIndex, max_chunk_size: Option<u64>) -> Self {
        Chunk {
            index,
            max_chunk_size: Self::get_chunk_size(max_chunk_size),
            filename: Self::index_to_name(index),
        }
    }
    /// Tries to open existing chunk with specified index.
    /// Returns an error when the chunk with such index does not exist.
    /// # Arguments
    /// * `index` - an index of the chunk (`0.db3`, `1.db3`, etc.)
    /// * `max_chunk_size` - max chunk size in bytes. Default is 1GB.
    /// # Errors
    /// If any IO error is encountered, its variant will be returned. The most common error should be non-existing file.
    /// If chunk with specified index is too big, error will be returned.
    pub fn try_open(index: ChunkIndex, max_chunk_size: Option<u64>) -> ChunkResult<Self> {
        let chunk = Chunk::new(index, max_chunk_size);
        chunk.validate_chunk_size()?;

        Ok(chunk)
    }

    pub fn try_create(index: ChunkIndex, max_chunk_size: Option<u64>) -> ChunkResult<Self> {
        let chunk = Chunk::new(index, max_chunk_size);
        File::create(&chunk.filename)?;
        Ok(chunk)
    }

    /// Tries to open already existing chunk starting from `index`. If chunk is larger than the `max_chunk_size`, tries to open the next one.
    /// # Errors
    /// If any IO error (except [`NotFound`]) is encountered the function will return immediately
    pub fn try_new_from(index: ChunkIndex, max_chunk_size: Option<u64>) -> ChunkResult<Self> {
        let mut index = index;
        loop {
            let chunk = Self::try_open(index, max_chunk_size);
            match chunk {
                Err(e) => match e {
                    ChunkError::ChunkFileDoesNotExist => {
                        return Self::try_create(index, max_chunk_size)
                    }
                    ChunkError::ChunkTooLarge => {
                        index += 1;
                        continue;
                    }
                    _ => return Err(e),
                },
                Ok(val) => return Ok(val),
            };
        }
    }

    /// Converts chunk name (`0.db3`) to the chunk index
    pub fn name_to_index(chunk_name: String) -> ChunkIndex {
        let index_str = chunk_name.replace(CHUNK_EXT, "");

        index_str.parse::<ChunkIndex>().unwrap()
    }

    pub fn index_to_name(chunk_index: ChunkIndex) -> String {
        format!("{}{}", chunk_index, CHUNK_EXT)
    }

    fn validate_chunk_size(&self) -> ChunkResult<()> {
        self.file_exists()?;
        let file = self.get_file(FileMode::Write)?;

        if file.metadata()?.len() >= self.max_chunk_size {
            Err(ChunkError::ChunkTooLarge)
        } else {
            Ok(())
        }
    }

    fn get_file(&self, mode: FileMode) -> io::Result<File> {
        match mode {
            FileMode::Append => OpenOptions::new().append(true).open(&self.filename),
            FileMode::Write => OpenOptions::new().write(true).open(&self.filename),
            FileMode::Read => OpenOptions::new().read(true).open(&self.filename),
        }
    }

    fn file_exists(&self) -> ChunkResult<()> {
        if Path::new(&self.filename).exists() {
            Ok(())
        } else {
            Err(ChunkError::ChunkFileDoesNotExist)
        }
    }

    fn get_chunk_size(chunk_size: Option<u64>) -> u64 {
        chunk_size.unwrap_or(MAX_CHUNK_SIZE)
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use super::*;
    use crate::in_temp_dir;
    use rusty_fork::rusty_fork_test;

    mod try_new {

        use super::*;
        rusty_fork_test! {

        #[test]
        fn no_chunks_exist_should_create_zero_chunk() {
            in_temp_dir!({
                let chunk = some_chunk(Some(1));

                assert_eq!(chunk.index, 0);
                assert!(exists_index(0))
            });
        }
        }

        rusty_fork_test! {
            #[test]
            fn chunk_exists_and_exceeds_limit_should_increment_index_and_create_new_chunk() {
                in_temp_dir!({
                    File::create("0.db3").unwrap().write_all(b"buf").unwrap();
                    let chunk = some_chunk(Some(1));
                    assert_eq!(chunk.index, 1);
                    assert!(exists_index(1))
                });
            }
        }

        rusty_fork_test! {
            #[test]
            fn chunk_exists_not_exceeds_limit_should_open_without_creating_new() {
                in_temp_dir!({
                    File::create("0.db3").unwrap().write_all(b"buf").unwrap();
                    let chunk = some_chunk(Some(99999));
                    assert_eq!(chunk.index, 0);
                    assert!(exists_index(0))
                });
            }
        }

        rusty_fork_test! {
            #[test]
            fn try_new_from_starts_from_provided_index() {
                in_temp_dir!({
                    File::create("1.db3").unwrap().write_all(b"buf").unwrap();
                    let chunk = Chunk::try_new_from(1, Some(1)).unwrap();
                    assert_eq!(chunk.index, 2);
                    assert!(exists_index(2));
                });
            }
        }
    }
    mod append {
        use super::*;
        rusty_fork_test! {
            #[test]
            fn append_chunk_size_exceeded_returns_error() {
                in_temp_dir!({
                    let mut chunk = some_chunk(Some(1));
                    chunk.try_append_data(b"test data").unwrap(); //exceeding the limit during the first write is okay
                    let err = chunk
                        .try_append_data(b"other data")
                        .expect_err("Should exceed limit of one byte");

                    assert!(matches!(err, ChunkError::ChunkTooLarge));
                });
            }
        }

        rusty_fork_test! {
            #[test]
            fn append_appends() {
                in_temp_dir!({
                    let mut chunk = some_chunk(Some(9999));
                    chunk.try_append_data(b"test").unwrap();
                    chunk.try_append_data(b"_data").unwrap();

                    let contents = fs::read_to_string("0.db3").unwrap();
                    assert_eq!(contents, "test_data");
                });
            }
        }
        // rusty_fork_test! {
        #[test]
        fn append_returns_correct_offset() {
            in_temp_dir!({
                let mut chunk = some_chunk(Some(9999));
                chunk.try_append_data(b"test").unwrap();
                let offset = chunk.try_append_data(b"test").unwrap();

                assert_eq!(offset, 4);
            });
        }
        //}
    }

    mod extend {
        use super::*;

        rusty_fork_test! {
            #[test]
            fn extend_should_create_new_file() {
                in_temp_dir!({
                    let chunk = some_chunk(Some(1));
                    let new_chunk = chunk.create_extended().unwrap();

                    assert_eq!(new_chunk.index, 1)
                });
            }
        }
    }

    mod open {
        use super::*;

        rusty_fork_test! {
            #[test]
            fn try_open_should_return_error_if_max_size_exceeded() {
                in_temp_dir!({
                    File::create("0.db3").unwrap().write_all(b"buf").unwrap();
                    let chunk = Chunk::try_open(0, Some(1));
                    assert!(matches!(chunk.unwrap_err(), ChunkError::ChunkTooLarge))
                });
            }
        }

        rusty_fork_test! {
            #[test]
            fn try_open_should_open_chunk_if_size_not_exceeded() {
                in_temp_dir!({
                    File::create("0.db3").unwrap().write_all(b"buf").unwrap();

                    let chunk = Chunk::try_open(0, Some(9999)).unwrap();
                    assert_eq!(chunk.index, 0);
                });
            }
        }
    }

    mod write {
        use super::*;

        rusty_fork_test! {
            #[test]
            fn try_write_data_should_write_at_given_offset() {
                in_temp_dir!({
                    File::create("0.db3").unwrap().write_all(b"buffer").unwrap();
                    let mut chunk = Chunk::open_without_sizecheck(0).unwrap();

                    chunk.try_write_data(b"i", 1).unwrap();

                    let file_contents = fs::read_to_string("0.db3").unwrap();
                    assert_eq!(file_contents, "biffer")
                });
            }
        }
    }

    mod read {
        use super::*;

        rusty_fork_test! {
            #[test]
            fn read_data_respects_offset_and_length() {
                in_temp_dir!({
                    File::create("0.db3").unwrap().write_all(&[1,2,3,4,5,6,7]).unwrap();
                    let chunk = Chunk::open_without_sizecheck(0).unwrap();

                    let res = chunk.read_data(3, 3).unwrap();
                    assert_eq!(res, vec![4,5,6]);
                });
            }
        }
    }

    mod remove {
        use super::*;

        rusty_fork_test! {
            #[test]
            fn remove_data_replaces_data_in_given_offset_and_length_with_zeros() {
                in_temp_dir!({
                    File::create("0.db3").unwrap().write_all(&[0,1,2,3,4,5,6,7]).unwrap();
                    let mut chunk = Chunk::open_without_sizecheck(0).unwrap();

                    chunk.remove_data(2, 3).unwrap();
                    let file_contents = fs::read("0.db3").unwrap();
                    assert_eq!(file_contents, vec![0,1,0,0,0,5,6,7]);
                });
            }
        }
    }

    fn exists_index(index: ChunkIndex) -> bool {
        return Path::new(&format!("{}.db3", index)).exists();
    }

    fn some_chunk(max_chunk_size: Option<u64>) -> Chunk {
        Chunk::try_new(max_chunk_size).unwrap()
    }
}
