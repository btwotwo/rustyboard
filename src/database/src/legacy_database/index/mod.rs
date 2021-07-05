pub mod db_post_ref;
pub mod serialized;
use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use self::{
    db_post_ref::{DbPostRef, DbPostRefHash},
    serialized::{IndexCollection, PostHashes},
};

pub type DbRefHashMap = HashMap<Rc<DbPostRefHash>, DbPostRef>;
pub type RepliesHashMap = HashMap<Rc<DbPostRefHash>, Vec<Rc<DbPostRefHash>>>;
pub type OrderedHashes = Vec<Rc<DbPostRefHash>>;
pub type DeletedPosts = HashSet<Rc<DbPostRefHash>>;
pub type FreeSpaceHashes = HashSet<Rc<DbPostRefHash>>;

#[derive(Default)]
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
    pub free: FreeSpaceHashes,
}

impl Reference {
    pub fn new(index_collection: IndexCollection) -> Reference {
        let mut refr = Reference::default();
        refr.ordered.reserve(index_collection.indexes.len());

        for ser_post in index_collection.indexes {
            let (raw_hashes, data) = ser_post.split();
            refr.put_ref(raw_hashes, data);
        }

        refr
    }

    pub fn put_ref(&mut self, hashes: PostHashes, post: DbPostRef) {
        let hash_rc = Rc::new(hashes.hash);
        let parent_rc = self.get_parent_rc(hashes.parent);

        if post.deleted {
            self.deleted.insert(hash_rc.clone());

            if post.length > 0 {
                self.free.insert(hash_rc.clone());
            }
        }

        self.refs.insert(hash_rc.clone(), post);

        let parent_post_replies = self
            .reply_refs
            .entry(parent_rc.clone())
            .or_insert_with(Vec::new);

        parent_post_replies.push(hash_rc.clone());
        self.ordered.push(hash_rc.clone());
    }

    fn get_parent_rc(&self, parent: DbPostRefHash) -> Rc<DbPostRefHash> {
        let rc = Rc::new(parent);
        let kv = self.refs.get_key_value(&rc);

        match kv {
            Some((key, _)) => Rc::clone(key),
            None => rc,
        }
    }
}

#[cfg(test)]
mod tests;
