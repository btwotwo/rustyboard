use std::{collections::HashMap, rc::Rc};

pub type DbPostRefHash = String;

pub struct DbPostRef {
    pub offset: u64,
    pub length: u64,
    pub deleted: bool,
    pub chunk_name: String,
}
pub type DbRefHashMap = HashMap<Rc<DbPostRefHash>, DbPostRef>;
pub type RepliesHashMap = HashMap<Rc<DbPostRefHash>, Vec<Rc<DbPostRefHash>>>;
pub type Ordered = Vec<Rc<DbPostRefHash>>;
