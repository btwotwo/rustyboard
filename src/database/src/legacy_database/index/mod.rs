pub mod db_post_ref;
pub mod serialized;
use std::{collections::{HashMap, HashSet}, rc::Rc};

use self::{db_post_ref::{DbPostRef, DbPostRefHash}, serialized::{IndexCollection, RawHashes}};


pub type DbRefHashMap = HashMap<Rc<DbPostRefHash>, DbPostRef>;
pub type RepliesHashMap = HashMap<Rc<DbPostRefHash>, Vec<Rc<DbPostRefHash>>>;
pub type OrderedHashes = Vec<Rc<DbPostRefHash>>;
pub type DeletedPosts = HashSet<Rc<DbPostRefHash>>;
pub type FreePostSpace = HashSet<Rc<DbPostRefHash>>;

pub struct Reference {
    ///[HashMap] of post references. `Key`is hash of the post, and `value` is [DbPostRef] 
    pub refs: DbRefHashMap,

    ///[HashMap] of post replies. `Key` is hash of the post, and `value` is [Vec] of [DbPostRef]
    pub reply_refs: RepliesHashMap,

    /// Posts in the same order as they are in the index collection
    pub ordered: OrderedHashes,

    ///Post hashes which were deleted from the database
    pub deleted: DeletedPosts,

    ///Post hashes which are marked as deleted and their space is not used now
    pub free: FreePostSpace
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
        let mut deleted = DeletedPosts::new();
        let mut free = FreePostSpace::new();

        let mut hashes_set = HashSet::new();

        for ser_post in index_collection.indexes {
            let (raw_hashes, data) = ser_post.split();

            let rc_hashes = Self::get_post_and_parent_rcs(raw_hashes, &mut hashes_set);
            let post_hash = rc_hashes.post_hash;
            let parent_hash = rc_hashes.parent_hash;

            if data.deleted {
                deleted.insert(Rc::clone(&post_hash));

                if data.length > 0 {
                    free.insert(Rc::clone(&post_hash));
                }
            }
            
            refs.insert(Rc::clone(&post_hash), data);

            let parent_post_replies = reply_refs.entry(parent_hash).or_insert_with(Vec::new);
            parent_post_replies.push(Rc::clone(&post_hash));

            ordered.push(Rc::clone(&post_hash))

        }

        Reference {
            refs,
            reply_refs,
            ordered,
            deleted,
            free
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

#[cfg(test)]
mod tests;
