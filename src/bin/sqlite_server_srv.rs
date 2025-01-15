use std::env;
use std::path::Path;
use std::sync::Arc;

use actix_web::{web, App, HttpServer};
use futures::{
    channel::mpsc::{channel, Receiver},
    SinkExt, StreamExt,
};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use papaya::HashMap;
use sqlite_server::cli::util::get_flag_val;
use sqlite_server::core::constants::{cli, queries};
use sqlite_server::core::db::{fetch_all_as_json, AppliedQuery};
use sqlite_server::core::state::{AppState, User};
use sqlx::SqlitePool;

include!(concat!(env!("OUT_DIR"), "/gen.rs"));

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

async fn async_watch(user_db: String, app_data: web::Data<AppState>) -> notify::Result<()> {
    let (mut watcher, mut rx) = async_watcher()?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(Path::new(&user_db).as_ref(), RecursiveMode::NonRecursive)?;

    while let Some(res) = rx.next().await {
        match res {
            Ok(ev) => {
                let pool = SqlitePool::connect(&format!("sqlite:{}", user_db))
                    .await
                    .unwrap();

                let users =
                    fetch_all_as_json(AppliedQuery::new(queries::GET_USERS_AND_ACCESS), &pool)
                        .await
                        .unwrap();

                pool.close().await;

                let users_map = Arc::clone(&app_data.users);
                let users_map_pin = users_map.pin();

                println!("BEFORE______________\n{:?}", users_map);
                users_map_pin.clear();

                users.iter().for_each(move |x| {
                    let username = x.get("username").unwrap();
                    let username_hash = x.get("username_hash").unwrap();
                    let username_password_hash = x.get("username_password_hash").unwrap();
                    let databases = x.get("databases").unwrap();
                    let db_access_rights = HashMap::new();
                    let db_access_rights_pin = db_access_rights.pin();

                    if databases.is_array() {
                        databases.as_array().unwrap().iter().for_each(|obj| {
                            serde_json::from_value::<std::collections::HashMap<String, u8>>(
                                obj.clone(),
                            )
                            .unwrap()
                            .iter()
                            .for_each(|(k, v)| {
                                db_access_rights_pin.insert(k.clone(), *v);
                            });
                        });
                    }

                    users_map_pin.insert(
                        String::from(username_hash.as_str().unwrap()),
                        User {
                            username: String::from(username.as_str().unwrap()),
                            username_password_hash: String::from(
                                username_password_hash.as_str().unwrap(),
                            ),
                            db_access_rights: db_access_rights.clone(),
                        },
                    );
                });
                println!("AFTER______________\n{:?}", users_map);
                println!("{ev:?}")
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut port = cli::DEFAULT_PORT;
    let mut db_max_conn = cli::DEFAULT_DB_MAX_CONN;
    let mut db_max_idle_time = cli::DEFAULT_DB_MAX_IDLE_TIME;

    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        port = get_flag_val::<u16>(&args, cli::PORT_FLAG).unwrap_or(cli::DEFAULT_PORT);
        db_max_conn =
            get_flag_val::<u32>(&args, cli::DB_MAX_CONN_FLAG).unwrap_or(cli::DEFAULT_DB_MAX_CONN);
        db_max_idle_time = get_flag_val::<u64>(&args, cli::DB_MAX_IDLE_TIME_FLAG)
            .unwrap_or(cli::DEFAULT_DB_MAX_IDLE_TIME);
    }

    let root_dir = Path::new(ROOT_DIR);
    let cfg_path = root_dir.join("cfg");
    let consumer_db_path = root_dir.join("db");
    let user_db_path = cfg_path.join(USER_DB_HASH);
    let user_db = format!("{}/{}.db", user_db_path.to_str().unwrap(), USER_DB_HASH);

    let app_data = web::Data::new(AppState {
        db_connections: Arc::new(HashMap::new()),
        users: Arc::new(HashMap::new()),
        db_max_connections: db_max_conn,
        db_max_idle_time,
        db_path: String::from(consumer_db_path.to_str().unwrap()),
    });

    let app_data_c = app_data.clone();
    let pool = SqlitePool::connect(&format!("sqlite:{}", user_db.clone()))
        .await
        .unwrap();

    let users = fetch_all_as_json(AppliedQuery::new(queries::GET_USERS_AND_ACCESS), &pool)
        .await
        .unwrap();

    pool.close().await;

    let users_map = Arc::clone(&app_data.users);
    let users_map_pin = users_map.pin();

    users.iter().for_each(move |x| {
        let username = x.get("username").unwrap();
        let username_hash = x.get("username_hash").unwrap();
        let username_password_hash = x.get("username_password_hash").unwrap();
        let databases = x.get("databases").unwrap();
        let db_access_rights = HashMap::new();
        let db_access_rights_pin = db_access_rights.pin();

        if databases.is_array() {
            databases.as_array().unwrap().iter().for_each(|obj| {
                serde_json::from_value::<std::collections::HashMap<String, u8>>(obj.clone())
                    .unwrap()
                    .iter()
                    .for_each(|(k, v)| {
                        db_access_rights_pin.insert(k.clone(), *v);
                    });
            });
        }

        users_map_pin.insert(
            String::from(username_hash.as_str().unwrap()),
            User {
                username: String::from(username.as_str().unwrap()),
                username_password_hash: String::from(username_password_hash.as_str().unwrap()),
                db_access_rights: db_access_rights.clone(),
            },
        );
    });

    let srv = HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .configure(sqlite_server::web::controller::init_db_controller)
            .configure(sqlite_server::web::controller::init_token_test_controller)
    })
    .bind(("127.0.0.1", port))
    .unwrap()
    .run();

    println!(
        ":::SERVER START:::\n127.0.0.1\nPORT={}\nDB_MAX_CONN={}\nDB_MAX_IDLE_TIME={}",
        port, db_max_conn, db_max_idle_time
    );

    actix_web::rt::spawn(async {
        let _ = async_watch(user_db, app_data_c).await;
    });
    
    srv.await
}
