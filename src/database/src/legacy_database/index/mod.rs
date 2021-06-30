mod db_post_ref;
mod serialized;
use std::{collections::{HashMap, HashSet}, fs::File, hash::Hash, io::BufReader, rc::Rc};

use self::{db_post_ref::{DbPostRefHash, DbRefHashMap, RepliesHashMap}, serialized::IndexCollection};

const INDEX_FILENAME: &str = "index-3.json";
const DIFF_FILENAME: &str = "diff-3.list";
pub struct Reference {
    refs: DbRefHashMap,
    reply_refs: RepliesHashMap,
    ordered: Vec<Rc<DbPostRefHash>>,
}

impl Reference {
    pub fn new(index: IndexCollection) -> std::io::Result<Reference> {
        let index_file = File::open(INDEX_FILENAME)?;
        let index_collection = serde_json::from_reader::<_, IndexCollection>(BufReader::new(index_file))?;

        let mut refs  = DbRefHashMap::new();
        let mut reply_refs  = RepliesHashMap::new();
        let hashes = HashSet::new();
        
        for ser_post in index_collection.indexes {
            let (hash, data) = ser_post.split();
            let hash_rc = Rc::new(hash);

            refs.insert(Rc::clone(&hash_rc), data);

            let key = Rc::clone(&hash_rc);
            let parent_post_replies = reply_refs.entry(key).or_insert(Vec::new());
        }

        todo!()

    }
}
