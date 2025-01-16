use std::env;
use std::path::Path;
use std::sync::Arc;

use actix_web::{web, App, HttpServer};
use papaya::HashMap;
use sqlite_server::core::constants::cli;
use sqlite_server::core::state::AppState;
use sqlite_server::{
    cli::util::get_flag_val,
    core::util::{async_watch, get_db_users, populate_app_state_users},
};

include!(concat!(env!("OUT_DIR"), "/gen.rs"));

const HOST: &str = "127.0.0.1";

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
    let db_users = get_db_users(&user_db).await;
    populate_app_state_users(db_users, &app_data);

    let srv = HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .configure(sqlite_server::web::controller::init_db_controller)
            .configure(sqlite_server::web::controller::init_token_test_controller)
    })
    .bind((HOST, port))
    .unwrap()
    .run();

    println!(
        "SERVER RUNNING @ {}:{}\ndb_max_conn={}\ndb_max_idle_time={}",
        HOST, port, db_max_conn, db_max_idle_time
    );

    actix_web::rt::spawn(async {
        let _ = async_watch(user_db, app_data_c).await;
    });

    srv.await
}
