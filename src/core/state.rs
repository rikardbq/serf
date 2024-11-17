use std::sync::Arc;

use sqlx::SqlitePool;

pub struct AppState {
    pub database_connections: Arc<papaya::HashMap<String, SqlitePool>>,
    pub usr: Arc<papaya::HashMap<String, Usr>>,
    pub db_max_conn: u32,
    pub db_max_idle_time: u64,
}

#[derive(Debug)]
pub struct Usr {
    pub u: String,
    pub up_hash: String,
    pub db_ar: std::collections::HashMap<String, u8>,
}
