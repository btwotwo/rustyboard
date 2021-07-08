use std::fs::File;
use std::fs::OpenOptions;
use std::io::ErrorKind::NotFound;
use std::io::Write;
use thiserror::Error;

pub const CHUNK_EXT: &str = ".db3";

#[allow(clippy::identity_op)] // For better readability
const MAX_CHUNK_SIZE: u64 = 1 * 1024 * 1024 * 1024; // 1 GB

pub type ChunkIndex = u64;
pub type Offset = u64;
#[derive(Debug)]
pub struct Chunk {
    pub index: ChunkIndex,
    file: File,
    max_chunk_size: u64,
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
}
pub type ChunkResult<T> = std::result::Result<T, ChunkError>;

impl Chunk {
    /// Tries to open existing chunk with specified index.
    /// Returns an error when the chunk with such index does not exist.
    /// # Arguments
    /// * `index` - an index of the chunk (`0.db3`, `1.db3`, etc.)
    /// * `max_chunk_size` - max chunk size in bytes. Default is 1GB.
    /// # Errors
    /// If any IO error is encountered, its variant will be returned. The most common error should be non-existing file.
    /// If chunk with specified index is too big, error will be returned.
    pub fn try_open(index: ChunkIndex, max_chunk_size: Option<u64>) -> ChunkResult<Self> {
        let max_chunk_size = Self::get_chunk_size(max_chunk_size);
        let filename = Self::get_filename(index);
        let file = OpenOptions::new().append(true).open(filename)?;
        let chunk = Chunk {
            index,
            file,
            max_chunk_size,
        };

        chunk.validate_chunk_size()?;

        Ok(chunk)
    }

    /// Tries to open existing chunk with specified index.
    ///
    /// **Warning!** This function doesn't check for chunk's size
    /// # Arguments
    /// * `index` - chunk's index ('0.db3', '1.db3'...)

    //TODO: tests
    pub fn open(index: ChunkIndex) -> ChunkResult<Self> {
        let filename = Self::get_filename(index);
        let file = OpenOptions::new().append(true).open(filename)?;

        Ok(Chunk {
            file,
            index,
            max_chunk_size: MAX_CHUNK_SIZE,
        })
    }

    pub fn try_create(index: ChunkIndex, max_chunk_size: Option<u64>) -> ChunkResult<Self> {
        let max_chunk_size = max_chunk_size.unwrap_or(MAX_CHUNK_SIZE);
        let filename = Self::get_filename(index);
        let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(filename)?;

        Ok(Chunk {
            index,
            file,
            max_chunk_size,
        })
    }

    /// Tries to open already existing chunk, starting from `0.db3`. If chunk is larger than the limit, tries to open the next one.
    /// # Errors
    /// If any IO error (except [`NotFound`]) is encountered the function will return immediately
    pub fn try_new(max_chunk_size: Option<u64>) -> ChunkResult<Self> {
        Self::try_new_from(0, max_chunk_size)
    }

    /// Tries to open already existing chunk starting from `index`. If chunk is larger than the `max_chunk_size`, tries to open the next one.
    /// # Errors
    /// If any IO error (except [`NotFound`]) is encountered the function will return immediately

    //TODO: tests
    pub fn try_new_from(index: ChunkIndex, max_chunk_size: Option<u64>) -> ChunkResult<Self> {
        let mut index = index;
        loop {
            let chunk = Self::try_open(index, max_chunk_size);
            match chunk {
                Err(e) => match e {
                    ChunkError::IoError { ref source } => match source.kind() {
                        NotFound => return Self::try_create(index, max_chunk_size),
                        _ => return Err(e),
                    },
                    ChunkError::ChunkTooLarge => {
                        index += 1;
                        continue;
                    }
                },
                Ok(val) => return Ok(val),
            };
        }
    }

    /// Creates a new chunk with incremented index
    pub fn extend(self) -> ChunkResult<Self> {
        let new_index = self.index + 1;
        let chunk = Self::try_create(new_index, Some(self.max_chunk_size))?;

        Ok(chunk)
    }

    /// Appends data to the chunk.
    /// # Errors
    /// If the chunk is too large, will return an error.
    pub fn try_append_data(&mut self, data: &[u8]) -> ChunkResult<Offset>
    {
        self.validate_chunk_size()?;
        let pos = self.file.stream_position()?;
        self.file.write_all(data.into())?;

        Ok(pos)
    }

        Ok(())
    }

    /// Converts chunk name (`0.db3`) to the chunk index
    pub fn name_to_index(chunk_name: String) -> ChunkIndex {
        let index_str = chunk_name.replace(CHUNK_EXT, "");

        index_str.parse::<ChunkIndex>().unwrap()
    }

    fn validate_chunk_size(&self) -> ChunkResult<()> {
        if self.file.metadata()?.len() >= self.max_chunk_size {
            Err(ChunkError::ChunkTooLarge)
        } else {
            Ok(())
        }
    }

    fn get_filename(index: ChunkIndex) -> String {
        format!("{}{}", index, CHUNK_EXT)
    }

    fn get_chunk_size(chunk_size: Option<u64>) -> u64 {
        chunk_size.unwrap_or(MAX_CHUNK_SIZE)
    }
}

#[cfg(test)]
mod tests {
    use std::{env::set_current_dir, fs, path::Path};
    use tempdir::TempDir;

    macro_rules! in_temp_dir {
        ($block:block) => {
            let tmpdir = TempDir::new("db").unwrap();
            set_current_dir(&tmpdir).unwrap();

            $block;
        };
    }

    use super::*;
    use rusty_fork::rusty_fork_test;

    mod try_new {
        use super::*;
        rusty_fork_test! {
            #[test]
            fn no_chunks_exist_should_create_zero_chunk() {
                in_temp_dir!({
                    let chunk = Chunk::try_new(Some(1)).unwrap();

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
                    let chunk = Chunk::try_new(Some(1)).unwrap();
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
                    let chunk = Chunk::try_new(Some(99999)).unwrap();
                    assert_eq!(chunk.index, 0);
                    assert!(exists_index(0))
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
                    let mut chunk = Chunk::try_new(Some(1)).unwrap();
                    chunk.try_append_data("test data").unwrap(); //if we exceed the limit during the first write it's okay
                    let err = chunk.try_append_data("other data").expect_err("Should exceed limit of one byte");

                    assert!(matches!(err, ChunkError::ChunkTooLarge));
                });
            }
        }
    }

    mod extend {
        use super::*;

        rusty_fork_test! {
            #[test]
            fn extend_should_create_new_file() {
                in_temp_dir!({
                    let chunk = Chunk::try_new(Some(1)).unwrap();
                    let new_chunk = chunk.extend().unwrap();

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

    fn exists_index(index: ChunkIndex) -> bool {
        return Path::new(&format!("{}.db3", index)).exists();
    }
}
