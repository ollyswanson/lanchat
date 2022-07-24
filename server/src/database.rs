use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub type Db = Arc<Mutex<HashMap<u32, ()>>>;

pub fn init_db() -> Db {
    Arc::new(Mutex::new(HashMap::new()))
}
