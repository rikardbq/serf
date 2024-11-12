use std::sync::Arc;

use actix_web::{web, App, HttpServer};
use sqlite_server::core::state::AppState;
use sqlite_server::core::util::{parse_usr_config, usr_config_buffer, USR_CONFIG_LOCATION};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let usr_config = parse_usr_config(usr_config_buffer(USR_CONFIG_LOCATION));
    let app_data = web::Data::new(AppState {
        database_connections: Arc::new(papaya::HashMap::new()),
        usr: Arc::new(usr_config),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .configure(sqlite_server::web::controller::init_database_controller)
            .configure(sqlite_server::web::controller::init_token_test_controller)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
