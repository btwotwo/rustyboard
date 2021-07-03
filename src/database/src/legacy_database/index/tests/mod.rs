use std::{rc::Rc};

use pretty_assertions::{assert_eq};

use crate::legacy_database::index::Reference;

use super::{db_post_ref::DbPostRef, serialized::{DbPostRefSerialized, IndexCollection}};
#[test]
fn when_passed_index_collection_should_create_valid_reference() {
    let ref_1 = some_raw_ref("1", "0", 1);
    let ref_2 = some_raw_ref("2", "1", 5);
    let ref_3 = some_raw_ref("3", "1", 10);

    let collection = IndexCollection {
        indexes: vec![ref_1, ref_2, ref_3],
    };

    let reference = Reference::new(collection);
    let rcs = vec![rc("1"), rc("2"), rc("3")];

    assert_eq!(reference.ordered, rcs);

    assert_eq!(reference.refs[&rc("1")], some_ref(1));
    assert_eq!(reference.refs[&rc("2")], some_ref(5));
    assert_eq!(reference.refs[&rc("3")], some_ref(10));

    assert_eq!(reference.reply_refs[&rc("0")], vec![rc("1")]);
    assert_eq!(reference.reply_refs[&rc("1")], vec![rc("2"), rc("3")]);

    assert!(reference.reply_refs.contains_key(&rc("2")) == false);
    assert!(reference.reply_refs.contains_key(&rc("3")) == false);
}

fn rc(str: &str) -> Rc<String> {
    Rc::new(str.to_string())
}

fn some_ref(length: u64) -> DbPostRef {
    DbPostRef {
        chunk_name: "0.db3".to_string(),
        deleted: false,
        length: length,
        offset: 1,
    }
}

fn some_raw_ref(hash: &str, parent: &str, length: u64) -> DbPostRefSerialized {
    DbPostRefSerialized {
        hash: hash.to_string(),
        reply_to: parent.to_string(),
        offset: 1,
        length,
        deleted: false,
        chunk_name: "0.db3".to_string(),
    }
}
