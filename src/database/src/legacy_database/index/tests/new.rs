use crate::tests::test_utils::*;

#[test]
fn when_passed_index_collection_should_create_valid_reference() {
    let ref_1 = some_raw_ref("1", "0", 1);
    let ref_2 = some_raw_ref("2", "1", 5);
    let ref_3 = some_raw_ref("3", "1", 10);

    let reference = collection(vec![ref_1, ref_2, ref_3]);
    let rcs = vec![rc("1"), rc("2"), rc("3")];

    assert_eq!(reference.ordered, rcs);

    assert_eq!(reference.refs[&rc("1")], some_ref(1, "0"));
    assert_eq!(reference.refs[&rc("2")], some_ref(5, "1"));
    assert_eq!(reference.refs[&rc("3")], some_ref(10, "1"));

    assert_eq!(reference.reply_refs[&rc("0")], vec![rc("1")]);
    assert_eq!(reference.reply_refs[&rc("1")], vec![rc("2"), rc("3")]);

    assert!(reference.reply_refs.contains_key(&rc("2")) == false);
    assert!(reference.reply_refs.contains_key(&rc("3")) == false);
}

#[test]
fn when_contains_deleted_post_should_add_to_deleted() {
    let ref1 = some_raw_ref("1", "0", 1);
    let mut ref2 = some_raw_ref("2", "0", 5);
    ref2.deleted = true;

    let reference = collection(vec![ref1, ref2]);
    let deleted_rc = rc("2");

    assert_eq!(reference.deleted.len(), 1);
    assert!(reference.deleted.contains(&deleted_rc));
}

#[test]
fn when_there_is_unused_space_should_add_post_hash_to_free() {
    let mut ref1 = some_raw_ref("1", "0", 0);
    ref1.deleted = true;

    let mut ref2 = some_raw_ref("2", "0", 10);
    ref2.deleted = true;

    let ref3 = some_raw_ref("3", "1", 10);

    let reference = collection(vec![ref1, ref2, ref3]);
    let free_rc = rc("2");

    assert_eq!(reference.free.len(), 1);
    assert!(reference.free.contains(&free_rc))
}

#[test]
fn when_creating_new_collection_should_not_add_anything_to_diff() {
    let ref_1 = some_raw_ref("1", "0", 5);
    let ref_2 = some_raw_ref("2", "0", 3);
    let ref_3 = some_raw_ref("3", "1", 10);

    let refr = collection_with_diff(vec![ref_1, ref_2, ref_3]);

    assert!(refr.diff.data.is_empty());
}

#[test]
fn when_creating_new_collection_should_create_new_data_from_diff() {
    let refr = collection_with_diff(vec![]);
    assert_eq!(refr.refs[&rc("1")], some_raw_ref("1", "0", 10).split().1);
    assert_eq!(refr.refs[&rc("2")], some_raw_ref("2", "1", 5).split().1);
    assert_eq!(refr.refs[&rc("3")], some_raw_ref("3", "1", 10).split().1);
}

#[test]
fn when_creating_new_collection_should_update_existing_data_from_diff() {
    let ref_1 = some_raw_ref("1", "0", 300);

    let refr = collection_with_diff(vec![ref_1]);

    assert_eq!(refr.refs[&rc("1")].length, 10);
}
