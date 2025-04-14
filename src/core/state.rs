use std::sync::Arc;

use papaya::{Guard, HashMap};
use serde::Deserialize;
use sqlx::SqlitePool;

pub type DatabaseConnections = Arc<HashMap<Arc<str>, SqlitePool>>;
pub type Users = Arc<HashMap<Arc<str>, User>>;

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

    pub fn db_connections_guard(&self) -> impl Guard + '_ {
        self.db_connections.guard()
    }

    pub fn get_user<'guard>(
        &self,
        username_hash: &str,
        guard: &'guard impl Guard,
    ) -> Option<&'guard User> {
        Arc::clone(&self.users).get(username_hash, guard)
    }

    pub fn get_db_connection<'guard>(
        &self,
        db_name: &str,
        guard: &'guard impl Guard,
    ) -> Option<&'guard SqlitePool> {
        Arc::clone(&self.db_connections).get(db_name, guard)
    }

    pub fn insert_db_connection<'guard>(
        &self,
        db_name: &str,
        db_connection: SqlitePool,
        guard: &'guard impl Guard,
    ) {
        let db_connections: Arc<HashMap<Arc<str>, SqlitePool>> = Arc::clone(&self.db_connections);
        db_connections.insert(Arc::from(db_name), db_connection, guard);
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
pub struct User {
    pub username: String,
    pub username_hash: String,
    pub username_password_hash: String,
    #[serde(skip)]
    pub db_access_rights: HashMap<Arc<str>, u8>,
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
