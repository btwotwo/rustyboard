pub mod db_post_ref;
pub mod diff;
pub mod serialized;
use std::{
    collections::{HashMap, HashSet},
    mem,
    rc::Rc,
    usize,
};

use crate::post::{Post, PostMessage};

use self::{
    db_post_ref::{DbPostRef, DbPostRefHash},
    diff::{Diff, DiffFileError},
    serialized::{DbPostRefSerialized, IndexCollection, PostHashes},
};

pub type DbRefHashMap = HashMap<DbPostRefHash, DbPostRef>;
pub type RepliesHashMap = HashMap<DbPostRefHash, Vec<DbPostRefHash>>;
pub type OrderedHashes = Vec<DbPostRefHash>;
pub type DeletedPosts = HashSet<DbPostRefHash>;
pub type FreeSpaceHashes = HashSet<DbPostRefHash>;

/// Post references collection
pub struct DbRefCollection<TDiff: Diff> {
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

    diff: TDiff,
}

impl<TDiff: Diff> DbRefCollection<TDiff> {
    /// Constructs reference collection from raw deserialized database references.
    pub fn new(index_collection: IndexCollection) -> Result<Self, DiffFileError> {
        let (diff, diff_collection) = TDiff::drain()?;
        let mut refr = DbRefCollection {
            diff,
            deleted: Default::default(),
            free: Default::default(),
            ordered: Default::default(),
            refs: Default::default(),
            reply_refs: Default::default(),
        };
        refr.ordered.reserve(index_collection.indexes.len());

        refr.apply_serialized_posts(index_collection.indexes);
        refr.apply_serialized_posts(diff_collection);

        Ok(refr)
    }

    /// Puts post into the database reference collection.

    //todo: wrap into result and don't allow put duplicate non-deleted posts
    pub fn put_post(&mut self, post: Post) -> (DbPostRefHash, PostMessage) {
        let post_bytes = post.get_message_bytes();
        let hashes = PostHashes {
            hash: DbPostRefHash::new(post.hash),
            parent: DbPostRefHash::new(post.reply_to)
        };

        let mut post_ref = DbPostRef {
            chunk_settings: None,
            deleted: false,
            length: post_bytes.len() as u64,
            parent_hash: hashes.parent.clone(),
        };

        self.put_ref_into_free_chunk(&mut post_ref, &post_bytes);
        self.upsert_ref(&hashes, post_ref);
        self.diff
            .append(&hashes, self.refs.get(&hashes.hash).unwrap());

        (hashes.hash, post.message)
    }

    pub fn get_ref_mut(&mut self, hash: &str) -> Option<&mut DbPostRef> {
        self.refs.get_mut(&hash.to_string())
    }

    pub fn get_ref(&self, hash: &str) -> Option<&DbPostRef> {
        self.refs.get(&hash.to_string())
    }

    pub fn ref_exists(&self, hash: &str) -> bool {
        self.refs.contains_key(&hash.to_string())
    }

    pub fn ref_deleted(&self, hash: &str) -> bool {
        self.get_ref(hash).map_or(false, |val| val.deleted)
    }

    fn put_ref_into_free_chunk(&mut self, post_ref: &mut DbPostRef, post_bytes: &[u8]) {
        let opt_hash = self.find_free_ref(post_bytes);
        let free_ref_hash = match opt_hash {
            Some(it) => it,
            _ => return,
        };
        let free_ref = self.refs.get_mut(&free_ref_hash).unwrap();
        let free_chunk_settings = mem::replace(&mut free_ref.chunk_settings, None);
        post_ref.chunk_settings = free_chunk_settings;
        free_ref.length = 0;

        self.free.remove(&free_ref_hash);
        self.diff
            .append(
                &PostHashes {
                    hash: free_ref_hash,
                    parent: free_ref.parent_hash.clone(),
                },
                &free_ref,
            )
            .unwrap();
    }

    /// Puts post reference to the `refs`, `reply_refs`, and `deleted` if post was deleted.
    /// If ref is already in the collection, updates it and updates diff
    fn upsert_ref(&mut self, hashes: &PostHashes, post: DbPostRef) {
        let hash_rc = &hashes.hash;
        let parent_rc = self.get_parent_rc(Rc::clone(&hashes.parent));

        let is_presented = self.refs.contains_key(hash_rc);
        let parent_post_replies = self.reply_refs.entry(parent_rc).or_insert_with(Vec::new);
        if !is_presented {
            parent_post_replies.push(hash_rc.clone());
            self.ordered.push(hash_rc.clone());
        }

        if post.deleted {
            self.deleted.insert(hash_rc.clone());

            if post.length > 0 {
                self.free.insert(hash_rc.clone());
            } else if post.length == 0 {
                self.free.remove(hash_rc);
            }
        } else {
            self.deleted.remove(hash_rc);
            self.free.remove(hash_rc);
        }

        self.refs.insert(hash_rc.clone(), post);
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

    fn apply_serialized_posts(&mut self, posts: Vec<DbPostRefSerialized>) {
        for ser_post in posts {
            let (raw_hashes, data) = ser_post.split();
            self.upsert_ref(&raw_hashes, data);
        }
    }
}

#[cfg(test)]
mod tests;
