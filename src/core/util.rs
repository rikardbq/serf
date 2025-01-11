use core::str;
use std::sync::Arc;
use std::time::Duration;

use actix_web::web;
use papaya::Guard;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;

use crate::core::error::{DatabaseNotExistError, SerfError};

use super::error::Error;
use super::state::AppState;

pub async fn get_or_insert_db_connection<'a>(
    db_connections_guard: &'a impl Guard,
    data: &'a web::Data<AppState>,
    db_name: &'a str,
) -> Result<&'a SqlitePool, Error<'a>> {
    let db_connections: Arc<papaya::HashMap<String, SqlitePool>> = Arc::clone(&data.db_connections);

    if !db_connections.contains_key(db_name, db_connections_guard) {
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
            db_connections.insert(db_name.to_owned(), pool, db_connections_guard);
        } else {
            return Err(DatabaseNotExistError::default());
        }
    }

    Ok(db_connections.get(db_name, db_connections_guard).unwrap())
}
