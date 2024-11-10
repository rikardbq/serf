use std::sync::{ Arc, Mutex };
use std::collections::HashMap;

pub struct AppState<T> {
    pub database_connections: Arc<Mutex<HashMap<String, T>>>,
    pub usr: Arc<Mutex<HashMap<String, Usr>>>,
}

#[derive(Debug)]
pub struct Usr {
    pub u: String,
    pub up_hash: String,
    pub db_ar: HashMap<String, u8>,
}
