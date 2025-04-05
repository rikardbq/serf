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

use super::state::AppState;
use super::{
    constants::queries,
    db::{fetch_all_as_json, AppliedQuery},
    error::{DatabaseNotExistError, SerfError},
    serf_proto::Error,
    state::User,
};

pub async fn get_or_insert_db_connection<'a>(
    db_connections_guard: &'a impl Guard,
    data: &'a web::Data<AppState>,
    db_name: &'a str,
) -> Result<&'a SqlitePool, Error> {
    let db_connections: Arc<papaya::HashMap<Arc<str>, SqlitePool>> =
        Arc::clone(&data.db_connections);

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
            db_connections.insert(Arc::from(db_name), pool, db_connections_guard);
        } else {
            return Err(DatabaseNotExistError::default());
        }
    }

    Ok(db_connections.get(db_name, db_connections_guard).unwrap())
}

pub async fn get_db_users(user_db: &str) -> JsonValue {
    let pool = SqlitePool::connect(&format!("sqlite:{}", user_db))
        .await
        .unwrap();
    let users = fetch_all_as_json(AppliedQuery::new(queries::GET_USERS_AND_ACCESS), &pool)
        .await
        .unwrap();

    pool.close().await;

    users
}

pub fn populate_app_state_users(db_users: JsonValue, app_data: &web::Data<AppState>) {
    let app_state_users = Arc::clone(&app_data.users);
    let app_state_users_pin = app_state_users.pin();

    if app_state_users_pin.len() > 0 {
        app_state_users_pin.clear();
    }

    // TODO: FIX THIS
    if db_users.is_array() {
        db_users.as_array().unwrap().iter().for_each(|x| {
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

    // db_users_vec.iter().for_each(|x| {
    //     let user: User = serde_json::from_value(x.clone()).unwrap();
    //     let databases = x.get("databases").unwrap();
    //     let db_access_rights = HashMap::new();
    //     let db_access_rights_pin = db_access_rights.pin();

    //     if databases.is_array() {
    //         databases.as_array().unwrap().iter().for_each(|obj| {
    //             serde_json::from_value::<std::collections::HashMap<String, u8>>(obj.clone())
    //                 .unwrap()
    //                 .iter()
    //                 .for_each(|(k, v)| {
    //                     db_access_rights_pin.insert(k.clone(), *v);
    //                 });
    //         });
    //     }

    //     app_state_users_pin.insert(
    //         user.username_hash.clone(),
    //         User {
    //             username: user.username,
    //             username_hash: user.username_hash,
    //             username_password_hash: user.username_password_hash,
    //             db_access_rights: db_access_rights.clone(),
    //         },
    //     );
    // });
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
                let db_users = get_db_users(&file_path_string).await;
                populate_app_state_users(db_users, &app_data);
                println!("{ev:?}")
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}
