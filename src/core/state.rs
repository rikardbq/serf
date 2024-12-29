use std::sync::Arc;

use papaya::{Guard, HashMap, HashSet};
use sqlx::SqlitePool;

pub type DatabaseConnections = Arc<HashMap<String, SqlitePool>>;
pub type Users = Arc<HashMap<String, User>>;

#[derive(Debug)]
pub struct AppState {
    pub db_connections: DatabaseConnections,
    pub users: Users,
    pub db_max_connections: u32,
    pub db_max_idle_time: u64,
    pub db_path: String,
}

impl AppState {
    pub fn users_guard(&self) -> impl Guard + '_ {
        self.users.guard()
    }

    pub fn get_user<'guard>(
        &self,
        username_hash: &str,
        guard: &'guard impl Guard,
    ) -> Option<&'guard User> {
        Some(Arc::clone(&self.users).get(username_hash, guard)?)
    }
}

#[derive(Debug, Clone)]
pub struct User {
    pub username: String,
    pub username_password_hash: String,
    pub db_access_rights: HashMap<String, u8>,
}

impl User {
    pub fn guard(&self) -> impl Guard + '_ {
        self.db_access_rights.guard()
    }

    pub fn get_access_right(&self, db_name: &str) -> u8 {
        match self.db_access_rights.pin().get(db_name) {
            Some(ar) => *ar,
            None => 0,
        }
    }
}
