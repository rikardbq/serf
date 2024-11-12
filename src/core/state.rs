use std::sync::Arc;

use sqlx::SqlitePool;

pub struct AppState {
    pub database_connections: Arc<papaya::HashMap<String, SqlitePool>>,
    pub usr: Arc<papaya::HashMap<String, Usr>>,
}

#[derive(Debug)]
pub struct Usr {
    pub u: String,
    pub up_hash: String,
    pub db_ar: std::collections::HashMap<String, u8>,
}
