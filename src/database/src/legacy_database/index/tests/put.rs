use pretty_assertions::assert_eq;

use crate::{legacy_database::index::{db_post_ref::{ChunkSettings, DbPostRef}, serialized::PostHashes, tests::util::{rc, some_post}}, post::{Post, PostMessage}};

use super::util::{collection, collection_with_diff, some_raw_deleted_ref, some_raw_ref, some_raw_removed_ref};

#[test]
fn find_free_ref_should_not_return_when_chunk_settings_are_none() {
    let ref_1 = some_raw_ref("1", "0", 5);
    let ref_2 = some_raw_ref("2", "0", 5);

    let mut deleted_ref = some_raw_ref("3", "0", 100);
    deleted_ref.deleted = true;
    deleted_ref.chunk_name = None;

    let collection = collection(vec![ref_1, ref_2, deleted_ref]);

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

#[test]
fn put_post_should_return_empty_chunk_if_no_free_space_was_found() {
    let ref_1 = some_raw_ref("1", "0", 10);
    let ref_2 = some_raw_removed_ref("2", "0");
    let mut col = collection(vec![ref_1, ref_2]);

    let post = Post {
        hash: "3".to_string(),
        reply_to: "0".to_string(),
        message: message(),
    };

    let expected_db_ref = DbPostRef {
        chunk_settings: None,
        deleted: false,
        length: 7,
        parent_hash: rc("0"),
    };

    let (hash, _) = col.put_post(post);
    let db_ref = &col.refs[&hash];

    assert_eq!(db_ref, &expected_db_ref);
}

#[test]
fn put_post_should_return_free_chunk_name_and_offset_if_free_space_found() {
    let ref_1 = some_raw_ref("1", "0", 10);
    let mut deletd_ref = some_raw_deleted_ref("2", "0", 10);
    deletd_ref.chunk_name = Some("1.db3".to_string());
    deletd_ref.offset = 333;

    let removed_ref = some_raw_removed_ref("3", "0");

    let mut col = collection(vec![ref_1, deletd_ref, removed_ref]);

    let post = Post {
        hash: "4".to_string(),
        message: message(),
        reply_to: "0".to_string(),
    };

    let expected_db_ref = DbPostRef {
        chunk_settings: Some(ChunkSettings {
            chunk_index: 1,
            offset: 333,
        }),
        length: 7,
        deleted: false,
        parent_hash: rc("0"),
    };
    let (hash, _) = col.put_post(post);
    let db_ref = &col.refs[&hash];

    assert_eq!(db_ref, &expected_db_ref)
}

#[test]
fn put_post_when_inserts_into_free_space_should_remove_chunk_data_from_free_post_and_set_length_to_0(
) {
    let ref_1 = some_raw_ref("1", "0", 10);
    let mut deleted_ref = some_raw_deleted_ref("2", "0", 10);
    deleted_ref.chunk_name = Some("1.db3".to_string());
    deleted_ref.offset = 123;

    let mut col = collection(vec![ref_1, deleted_ref]);

    let post = Post {
        hash: "3".to_string(),
        message: message(),
        reply_to: "0".to_string(),
    };

    col.put_post(post);

    matches!(col.refs[&rc("2")].chunk_settings, None);
    assert_eq!(col.refs[&rc("2")].length, 0);
}

#[test]
fn put_post_should_update_diff() {
    let ref_1 = some_raw_ref("5", "0", 10);
    let mut coll = collection_with_diff(vec![ref_1]);
    let post = some_post("20", "1", "Test");
    let mut expected_ref = some_raw_ref("20", "1", 4);
    expected_ref.offset = 0;
    expected_ref.length = 4;
    expected_ref.chunk_name = None;
    
    coll.put_post(post);

    assert_eq!(coll.diff.data[0], expected_ref)
}

#[test]
fn upsert_ref_should_update_existing_data() {
    let ref_1 = some_raw_ref("1", "0", 10);
    let mut updated_ref = some_raw_ref("1", "0", 100);
    updated_ref.deleted = true;

    let mut coll = collection(vec![ref_1]);

    coll.upsert_ref(&PostHashes {
        hash: rc("1"),
        parent: rc("0")
    }, updated_ref.split().1);

    assert_eq!(coll.refs[&rc("1")].deleted, true);
    assert_eq!(coll.refs[&rc("1")].length, 100);
    assert!(coll.deleted.contains(&rc("1")));
}

fn message() -> PostMessage {
    PostMessage::new("message".to_string())
}
