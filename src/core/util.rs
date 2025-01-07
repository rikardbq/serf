use core::str;
use std::sync::Arc;
use std::time::Duration;

use actix_web::web;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;

use super::{
    constants::errors::{self, ErrorReason},
    state::AppState,
};

pub struct Error<'a> {
    pub message: &'a str,
    pub reason: Option<ErrorReason>,
}

impl<'a> Error<'a> {
    pub fn new(message: &'a str) -> Self {
        Error {
            message,
            reason: None,
        }
    }

    pub fn with_reason(self, reason: ErrorReason) -> Self {
        Error {
            reason: Some(reason),
            ..self
        }
    }
}

pub async fn get_db_connections<'a>(
    data: &'a web::Data<AppState>,
    db_name: &'a str,
) -> Result<SqlitePool, &'a str> {
    let db_connections_clone: Arc<papaya::HashMap<String, SqlitePool>> =
        Arc::clone(&data.db_connections);
    let db_connections = db_connections_clone.pin();

    if !db_connections.contains_key(db_name) {
        println!(
            "Database connection is not opened, trying to open database {}",
            db_name
        );
        if let Ok(pool) = SqlitePoolOptions::new()
            .max_connections(data.db_max_connections)
            .idle_timeout(Duration::from_secs(data.db_max_idle_time))
            .connect(&format!(
                "sqlite:{}/{}/{}.db",
                data.db_path, db_name, db_name
            ))
            .await
        {
            db_connections.insert(db_name.to_owned(), pool);
        } else {
            return Err(errors::ERROR_DATABASE_NOT_FOUND);
        }
    }

    Ok(db_connections.get(db_name).unwrap().to_owned())
}

