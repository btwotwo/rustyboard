pub mod db_post_ref;
pub mod serialized;
mod diff;
use std::{
    collections::{HashMap, HashSet},
    mem,
    rc::Rc,
    usize,
};

use crate::post::{Post, PostMessage};

use self::{
    db_post_ref::{DbPostRef, DbPostRefHash},
    serialized::{IndexCollection, PostHashes},
};

pub type DbRefHashMap = HashMap<DbPostRefHash, DbPostRef>;
pub type RepliesHashMap = HashMap<DbPostRefHash, Vec<DbPostRefHash>>;
pub type OrderedHashes = Vec<DbPostRefHash>;
pub type DeletedPosts = HashSet<DbPostRefHash>;
pub type FreeSpaceHashes = HashSet<DbPostRefHash>;

#[derive(Default)]
/// Post references collection
pub struct DbRefCollection {
    ///[HashMap] of post references. `Key`is hash of the post, and `value` is [DbPostRef]
    refs: DbRefHashMap,

    ///[HashMap] of post replies. `Key` is hash of the post, and `value` is [Vec] of [DbPostRef]
    reply_refs: RepliesHashMap,

    /// Posts in the same order as they are in the index collection
    ordered: OrderedHashes,

    ///Post hashes which were deleted from the database
    deleted: DeletedPosts,

    ///Post hashes which are marked as deleted and their space is not used now
    free: FreeSpaceHashes,
}

impl DbRefCollection {
    /// Constructs reference collection from raw deserialized database references.
    pub fn new(index_collection: IndexCollection) -> DbRefCollection {
        let mut refr = DbRefCollection::default();
        refr.ordered.reserve(index_collection.indexes.len());

        for ser_post in index_collection.indexes {
            let (raw_hashes, data) = ser_post.split();
            refr.put_ref(&raw_hashes, data);
        }

        refr
    }

    /// Puts post into the database reference collection.
    pub fn put_post(&mut self, post: Post) -> (DbPostRefHash, PostMessage) {
        let post_bytes = post.get_message_bytes();
        let mut post_ref = DbPostRef {
            chunk_settings: None,
            deleted: false,
            length: post_bytes.len() as u64,
        };

        let opt_hash = self.find_free_ref(&post_bytes);
        if let Some(free_ref_hash) = opt_hash {
            let free_ref = self.refs.get_mut(&free_ref_hash).unwrap();
            let free_chunk_settings = mem::replace(&mut free_ref.chunk_settings, None);
            post_ref.chunk_settings = free_chunk_settings;
            free_ref.length = 0;

            self.free.remove(&free_ref_hash);
            //todo: Update diff file!
        }

        let hashes = PostHashes {
            hash: DbPostRefHash::new(post.hash),
            parent: DbPostRefHash::new(post.reply_to),
        };

        self.put_ref(&hashes, post_ref);

        (hashes.hash, post.message)
    }

    pub fn get_ref_mut(&mut self, hash: &DbPostRefHash) -> Option<&mut DbPostRef> {
        self.refs.get_mut(hash)
    }

    /// Puts post reference to the `refs`, `reply_refs`, and `deleted` if post was deleted.
    ///
    /// If post was deleted, checks for the
    fn put_ref(&mut self, hashes: &PostHashes, post: DbPostRef) {
        let hash_rc = &hashes.hash;
        let parent_rc = self.get_parent_rc(Rc::clone(&hashes.parent));

        if post.deleted {
            self.deleted.insert(hash_rc.clone());

            if post.length > 0 {
                self.free.insert(hash_rc.clone());
            }
        }

        self.refs.insert(hash_rc.clone(), post);

        let parent_post_replies = self.reply_refs.entry(parent_rc).or_insert_with(Vec::new);

        parent_post_replies.push(hash_rc.clone());
        self.ordered.push(hash_rc.clone());
    }

    // Todo reuse the rest of free space
    fn find_free_ref(&self, post_bytes: &[u8]) -> Option<DbPostRefHash> {
        let post_length = post_bytes.len();
        let best = self.find_best_free_ref(post_length);

        match best {
            Some(hash) => {
                let clone = Rc::clone(&hash);
                Some(clone)
            }
            None => None,
        }
    }

    fn find_best_free_ref(&self, post_length: usize) -> Option<&DbPostRefHash> {
        let mut min = u64::MAX;
        let mut best: Option<&DbPostRefHash> = None;
        for hash in self
            .free
            .iter()
            .filter(|p| self.refs[*p].chunk_settings.is_some())
        {
            let free_item = &self.refs[hash];
            if free_item.length >= post_length as u64 {
                let diff = free_item.length - post_length as u64;

                if diff < min {
                    min = diff;
                    best = Some(hash);
                }
            }
        }

        best
    }

    fn get_parent_rc(&self, parent: DbPostRefHash) -> DbPostRefHash {
        let kv = self.refs.get_key_value(&parent);

        match kv {
            Some((key, _)) => Rc::clone(key),
            None => parent,
        }
    }
}

#[cfg(test)]
mod tests;
