use std::{collections::HashMap, fs::File, io::BufReader};

use serde::{Serialize, Deserialize};

const INDEX_FILENAME: &str = "index-3.json";
const DIFF_FILENAME: &str = "diff-3.list";

/// Reference of post messages, which are stored in chunks. This struct is serialized and written into
/// `index-3.json` to save message positions inside chunks.
#[derive(Serialize, Deserialize)]
pub struct DatabasePostRef {
    /// Post hash
    #[serde(rename = "h")]
    hash: String,

    /// Hash of the parent post
    #[serde(rename = "r")]
    reply_to: String,
    
    /// Offset in bytes from the start of the chunk
    #[serde(rename = "o")]
    offset: u64,

    /// Length of the post message in bytes
    #[serde(rename = "l")]
    length: u64,

    /// Is post deleted. If it's deleted, it won't be shown in list of the posts
    #[serde(rename = "d")]
    deleted: bool,

    /// Chunk filename
    #[serde(rename = "f")]
    chunk_name: String
}

/// A collection of database references
#[derive(Serialize, Deserialize)]
pub struct Index {
    indexes: Vec<DatabasePostRef>
}

pub struct Reference<'a> {
    refs: HashMap<String, DatabasePostRef>,
    reply_refs: HashMap<String, Vec<&'a DatabasePostRef>>,
    ordered: Index
}

impl Reference<'_> {
    pub fn new(index: Index) -> Self {
        Reference {
            refs: HashMap::new(),
            reply_refs: HashMap::new(),
            ordered: index,
        }
    }
}

pub fn reconstruct_reference() -> std::io::Result<Reference<'static>> {
    let index_file = File::open(INDEX_FILENAME)?;
    let index = serde_json::from_reader::<_, Index>(BufReader::new(index_file))?;
    let mut refer = Reference::new(index);

    for i in refer.ordered.indexes {
        refer.refs.insert(i.hash, i);
    }



    todo!()
}