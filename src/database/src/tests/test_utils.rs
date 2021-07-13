#[macro_export]
macro_rules! in_temp_dir {
    ($block:block) => {
        use std::env::set_current_dir;
        use tempdir::TempDir;
        let tmpdir = TempDir::new("db").unwrap();
        set_current_dir(&tmpdir).unwrap();

        $block;
    };
}
