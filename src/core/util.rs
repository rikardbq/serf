use core::str;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use actix_web::web;
use futures::{
    channel::mpsc::{channel, Receiver},
    SinkExt, StreamExt,
};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use papaya::{Guard, HashMap};
use serde_json::Value as JsonValue;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;

use super::{
    constants::queries,
    db::{fetch_all_as_json, AppliedQuery},
    error::{ResourceNotExistError, SerfError},
    serf_proto::Error,
    state::{AppState, User},
};

pub async fn create_db_connection(
    connection_string: &str,
    max_connections: u32,
    max_idle_time: u64,
) -> Result<SqlitePool, Error> {
    let error_msg = "Database does not exist";
    if connection_string.contains("..") {
        return Err(ResourceNotExistError::with_message(error_msg));
    }

    match SqlitePoolOptions::new()
        .max_connections(max_connections)
        .idle_timeout(Duration::from_secs(max_idle_time))
        .connect(connection_string)
        .await
    {
        Ok(pool) => Ok(pool),
        Err(_) => Err(ResourceNotExistError::with_message(error_msg)),
    }
}

pub async fn get_or_insert_db_connection<'a>(
    data: &'a web::Data<AppState>,
    db_name: &'a str,
    db_connections_guard: &'a impl Guard,
) -> Result<&'a SqlitePool, Error> {
    match data.get_db_connection(db_name, db_connections_guard) {
        Some(connection) => Ok(connection),
        None => {
            // ToDo: replace with real logs some day
            println!(
                "Database connection not open, trying to open for {}",
                db_name
            );

            match create_db_connection(
                &format!("sqlite:{}/{}/{}.db", data.db_path, db_name, db_name),
                data.db_max_connections,
                data.db_max_idle_time,
            )
            .await
            {
                Ok(conn) => {
                    // ToDo: replace with real logs some day
                    println!("Database connection opened for {}", db_name);
                    data.insert_db_connection(&db_name, conn, db_connections_guard);
                }
                Err(e) => {
                    return Err(e);
                }
            };

            Ok(data
                .get_db_connection(db_name, db_connections_guard)
                .unwrap())
        }
    }
}

pub async fn get_db_users(user_db: &str) -> Result<JsonValue, sqlx::error::Error> {
    let pool = SqlitePool::connect(&format!("sqlite:{}", user_db)).await?;
    let users = fetch_all_as_json(AppliedQuery::new(queries::GET_USERS_AND_ACCESS), &pool).await?;

    pool.close().await;

    Ok(users)
}

pub fn populate_app_state_users(db_users: JsonValue, app_data: &web::Data<AppState>) {
    let app_state_users = Arc::clone(&app_data.users);
    let app_state_users_pin = app_state_users.pin();

    if app_state_users_pin.len() > 0 {
        app_state_users_pin.clear();
    }

    if let Some(arr) = db_users.as_array() {
        arr.iter().for_each(|x| {
            let user: User = serde_json::from_value(x.clone()).unwrap();
            let databases = x.get("databases").unwrap();
            let db_access_rights = HashMap::new();
            let db_access_rights_pin = db_access_rights.pin();

            if databases.is_array() {
                databases.as_array().unwrap().iter().for_each(|obj| {
                    serde_json::from_value::<std::collections::HashMap<Arc<str>, u8>>(obj.clone())
                        .unwrap()
                        .iter()
                        .for_each(|(k, v)| {
                            db_access_rights_pin.insert(k.clone(), *v);
                        });
                });
            }

            app_state_users_pin.insert(
                Arc::from(user.username_hash.as_str()),
                User {
                    username: user.username,
                    username_hash: user.username_hash,
                    username_password_hash: user.username_password_hash,
                    db_access_rights: db_access_rights.clone(),
                },
            );
        });
    }
}

fn async_watcher() -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
    let (mut tx, rx) = channel(1);
    let watcher = RecommendedWatcher::new(
        move |res| {
            futures::executor::block_on(async {
                tx.send(res).await.unwrap();
            })
        },
        Config::default(),
    )?;

    Ok((watcher, rx))
}

pub async fn async_watch(
    file_path_string: String,
    app_data: web::Data<AppState>,
) -> notify::Result<()> {
    let (mut watcher, mut rx) = async_watcher()?;

    watcher.watch(
        Path::new(&file_path_string).as_ref(),
        RecursiveMode::NonRecursive,
    )?;

    while let Some(res) = rx.next().await {
        match res {
            Ok(ev) => {
                println!("{ev:?}");
                match get_db_users(&file_path_string).await {
                    Ok(val) => populate_app_state_users(val, &app_data),
                    Err(e) => eprintln!("watch error: {:?}", e),
                };
            }
            Err(e) => eprintln!("watch error: {:?}", e),
        }
    }

    Ok(())
}
