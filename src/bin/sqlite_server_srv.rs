use std::env;
use std::sync::Arc;

use actix_web::{web, App, HttpServer};
use sqlite_server::core::state::AppState;
use sqlite_server::core::util::{parse_usr_config, usr_config_buffer, USR_CONFIG_LOCATION};

include!(concat!(env!("OUT_DIR"), "/gen.rs"));

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut port = DEFAULT_PORT;
    let mut db_max_conn = DEFAULT_DB_MAX_CONN;
    let mut db_max_idle_time = DEFAULT_DB_MAX_IDLE_TIME;
    
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        for i in 0..args.len() - 1 {
            let flag = args[i].as_str();
            let flag_val = args[i + 1].as_str();
            let _ = match flag {
                "--port" => {
                    if !flag_val.starts_with("-") {
                        port = flag_val.parse::<u16>().unwrap_or_default();
                    }
                }
                "--db-max-conn" => {
                    if !flag_val.starts_with("-") {
                        db_max_conn = flag_val.parse::<u32>().unwrap_or_default();
                    }
                }
                "--db-max-idle-time" => {
                    if !flag_val.starts_with("-") {
                        db_max_idle_time = flag_val.parse::<u64>().unwrap_or_default();
                    }
                }
                _ => (),
            };
        }
    }

    let usr_config = parse_usr_config(usr_config_buffer(USR_CONFIG_LOCATION));
    let app_data = web::Data::new(AppState {
        database_connections: Arc::new(papaya::HashMap::new()),
        usr: Arc::new(usr_config),
        db_max_conn: db_max_conn,
        db_max_idle_time: db_max_idle_time
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
