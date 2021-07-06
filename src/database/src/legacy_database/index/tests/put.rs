use crate::legacy_database::index::DbRefCollection;

use super::util::some_raw_ref;

#[test]
fn find_free_ref_should_not_return_when_chunk_settings_are_none() {
    let mut ref_1 = some_raw_ref("1", "0", 5);
    // let ref_collection = DbRefCollection::new()
}
