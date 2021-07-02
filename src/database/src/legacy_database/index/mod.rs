mod db_post_ref;
mod serialized;
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    hash::Hash,
    io::BufReader,
    rc::Rc,
};

use self::{
    db_post_ref::{DbPostRefHash, DbRefHashMap, RepliesHashMap},
    serialized::{IndexCollection, RawHashes},
};

const INDEX_FILENAME: &str = "index-3.json";
const DIFF_FILENAME: &str = "diff-3.list";
pub struct Reference {
    refs: DbRefHashMap,
    reply_refs: RepliesHashMap,
    ordered: Vec<Rc<DbPostRefHash>>,
}
type RcHashSet = HashSet<Rc<DbPostRefHash>>;

struct RcHashes {
    post_hash: Rc<DbPostRefHash>,
    parent_hash: Rc<DbPostRefHash>,
}

impl Reference {
    pub fn new(index_collection: IndexCollection) -> Reference {
        let mut refs = DbRefHashMap::new();
        let mut reply_refs = RepliesHashMap::new();
        let mut ordered = Vec::with_capacity(index_collection.indexes.len());

        let mut hashes_set = HashSet::new();

        for ser_post in index_collection.indexes {
            let (raw_hashes, data) = ser_post.split();

            let rc_hashes = Self::get_post_and_parent_rcs(raw_hashes, &mut hashes_set);
            let post_hash = rc_hashes.post_hash;
            let parent_hash = rc_hashes.parent_hash;
            refs.insert(Rc::clone(&post_hash), data);

            let parent_post_replies = reply_refs.entry(parent_hash).or_insert(Vec::new());
            parent_post_replies.push(Rc::clone(&post_hash));

            ordered.push(Rc::clone(&post_hash))
        }

        Reference {
            ordered,
            refs,
            reply_refs,
        }
    }

    fn get_post_and_parent_rcs(hashes: RawHashes, hash_set: &mut RcHashSet) -> RcHashes {
        fn get_rc(hash: DbPostRefHash, hash_set: &mut RcHashSet) -> Rc<DbPostRefHash> {
            let rc = Rc::new(hash);
            if !hash_set.contains(&rc) {
                hash_set.insert(Rc::clone(&rc));
            }

            hash_set.get(&Rc::clone(&rc)).unwrap().clone()
        }
        let hash = get_rc(hashes.hash, hash_set);
        let parent = get_rc(hashes.parent, hash_set);

        RcHashes {
            post_hash: hash,
            parent_hash: parent,
        }
    }
}
