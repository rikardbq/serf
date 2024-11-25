use std::env;
use std::path::Path;
use std::sync::Arc;

use actix_web::{web, App, HttpServer};
use sqlite_server::core::constants::queries;
use sqlite_server::core::db::{fetch_all_as_json, AppliedQuery};
use sqlite_server::core::state::{AppState, Usr};
use sqlite_server::core::util::get_flag_val;
use sqlx::SqlitePool;

include!(concat!(env!("OUT_DIR"), "/gen.rs"));

const DEFAULT_PORT: u16 = 8080;
const DEFAULT_DB_MAX_CONN: u32 = 12;
const DEFAULT_DB_MAX_IDLE_TIME: u64 = 3600;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut port = DEFAULT_PORT;
    let mut db_max_conn = DEFAULT_DB_MAX_CONN;
    let mut db_max_idle_time = DEFAULT_DB_MAX_IDLE_TIME;

    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        port = get_flag_val::<u16>(&args, "--port").unwrap_or_default();
        db_max_conn = get_flag_val::<u32>(&args, "--db-max-conn").unwrap_or_default();
        db_max_idle_time = get_flag_val::<u64>(&args, "--db-max-idle-time").unwrap_or_default();
    }

    let root_dir = Path::new(ROOT_DIR);
    let cfg_path = root_dir.join("cfg");
    let consumer_db_path = root_dir.join("db");
    let user_db_full_path_string = format!("{}/{}", cfg_path.to_str().unwrap(), USER_DB_PATH);
    let user_db = format!("{}/{}.db", user_db_full_path_string, USER_DB_PATH);

    let pool = SqlitePool::connect(&format!("sqlite:{}", user_db))
        .await
        .unwrap();

    let users = fetch_all_as_json(AppliedQuery::new(queries::GET_USERS_AND_ACCESS), &pool)
        .await
        .unwrap();

    pool.close().await;

    let usr_map: papaya::HashMap<String, Usr> = papaya::HashMap::new();
    let usr_map_ref = usr_map.pin();

    users.iter().for_each(move |x| {
        let username = x.get("username").unwrap();
        let username_hash = x.get("username_hash").unwrap();
        let username_password_hash = x.get("username_password_hash").unwrap();
        let databases = x.get("databases").unwrap();
        let mut db_ar = std::collections::HashMap::new();

        if databases.is_array() {
            databases.as_array().unwrap().iter().for_each(|obj| {
                serde_json::from_value::<std::collections::HashMap<String, u8>>(obj.clone())
                    .unwrap()
                    .iter()
                    .for_each(|(k, v)| {
                        let _ = &mut db_ar.insert(k.clone(), *v);
                    });
            });
        }

        usr_map_ref.insert(
            String::from(username_hash.as_str().unwrap()),
            Usr {
                u: String::from(username.as_str().unwrap()),
                up_hash: String::from(username_password_hash.as_str().unwrap()),
                db_ar: db_ar,
            },
        );
    });

    let app_data = web::Data::new(AppState {
        database_connections: Arc::new(papaya::HashMap::new()),
        usr: Arc::new(usr_map),
        db_max_conn: db_max_conn,
        db_max_idle_time: db_max_idle_time,
        db_path: String::from(consumer_db_path.to_str().unwrap()),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .configure(sqlite_server::web::controller::init_database_controller)
            .configure(sqlite_server::web::controller::init_token_test_controller)
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}
