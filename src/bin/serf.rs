use std::env;
use std::path::Path;
use std::sync::Arc;

use actix_web::http::header::{ContentType, Header};
use actix_web::web::{Payload, PayloadConfig};
use actix_web::{web, App, HttpRequest, HttpServer};
use mime::Mime;
use papaya::HashMap;
use serf::core::constants::cli;
use serf::core::state::AppState;
use serf::{
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
    let mut db_max_lifetime = cli::DEFAULT_DB_MAX_LIFETIME;

    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        port = get_flag_val::<u16>(&args, cli::PORT_FLAG).unwrap_or(cli::DEFAULT_PORT);
        db_max_conn =
            get_flag_val::<u32>(&args, cli::DB_MAX_CONN_FLAG).unwrap_or(cli::DEFAULT_DB_MAX_CONN);
        db_max_idle_time = get_flag_val::<u64>(&args, cli::DB_MAX_IDLE_TIME_FLAG)
            .unwrap_or(cli::DEFAULT_DB_MAX_IDLE_TIME);
        db_max_lifetime = get_flag_val::<u64>(&args, cli::DB_MAX_LIFETIME_FLAG)
            .unwrap_or(cli::DEFAULT_DB_MAX_LIFETIME);
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
        db_max_lifetime,
        db_path: String::from(consumer_db_path.to_str().unwrap()),
    });
    let app_data_c = app_data.clone();
    match get_db_users(&user_db).await {
        Ok(val) => populate_app_state_users(val, &app_data),
        Err(e) => panic!("{e}"),
    };

    let srv = HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .app_data(
                web::PayloadConfig::new(100 * 1024 * 1024)
                    .mimetype("application/protobuf".parse::<mime::Mime>().unwrap()),
            )
            .configure(serf::web::controller::init_db_controller)
            .configure(serf::web::controller::init_health_controller)
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
