use crate::legacy_database::index::{
    serialized::IndexCollection, tests::util::rc, DbRefCollection,
};

use super::util::{collection, some_raw_deleted_ref, some_raw_ref, some_raw_removed_ref};

#[test]
fn find_free_ref_should_not_return_when_chunk_settings_are_none() {
    let ref_1 = some_raw_ref("1", "0", 5);
    let ref_2 = some_raw_ref("2", "0", 5);

    let mut deleted_ref = some_raw_ref("3", "0", 100);
    deleted_ref.deleted = true;
    deleted_ref.chunk_name = None;

    let collection = DbRefCollection::new(IndexCollection {
        indexes: vec![ref_1, ref_2, deleted_ref],
    });

    let free_ref_hash = collection.find_free_ref(&[1, 2, 3, 4, 5]);

    assert!(free_ref_hash.is_none());
}

#[test]
fn find_free_ref_should_return_valid_ref_when_there_is_enough_space() {
    let ref_1 = some_raw_ref("1", "0", 5);
    let deleted_ref = some_raw_deleted_ref("2", "0", 10);
    let removed_ref = some_raw_removed_ref("3", "0");

    let collection = collection(vec![ref_1, deleted_ref, removed_ref]);

    let free_hash = collection.find_free_ref(&[1, 2, 3, 4]);

    assert!(free_hash.is_some());
    assert_eq!(free_hash.unwrap(), rc("2"));
}

#[test]
fn find_free_ref_should_find_most_suitable() {
    let ref_1 = some_raw_ref("1", "0", 5);
    let deleted_1 = some_raw_deleted_ref("2", "0", 10);
    let deleted_2 = some_raw_deleted_ref("3", "0", 3);

    let col = collection(vec![ref_1, deleted_1, deleted_2]);

    let free_hash = col.find_free_ref(&[1, 2]);

    assert!(free_hash.is_some());
    assert_eq!(free_hash.unwrap(), rc("3"));
}
