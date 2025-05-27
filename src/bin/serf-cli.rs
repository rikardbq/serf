use std::env;

use serf::cli::util::{get_flag_val, DatabaseManager};
use serf::core::constants::cli;
use sqlx::{migrate::MigrateDatabase, Sqlite};

// DEFAULTS:
//  (root path) $HOME/.serf/
//  (cfg path) $HOME/.serf/cfg/
//  (users db path) $HOME/.serf/cfg/{hashed}/
//      then {hashed}.db file is the users db
//  (db paths) $HOME/.serf/db/{hashed}/
//      folder is a sha256 hash of the db name
//      containing {hashed}.db file
#[tokio::main]
async fn main() -> std::io::Result<()> {
    let database_manager = DatabaseManager::new();
    if !Sqlite::database_exists(&database_manager.user_db_full_path_string)
        .await
        .unwrap_or(false)
    {
        database_manager.init().await;
        println!("INITIAL SETUP");
    }

    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let cmd_one = args[1].as_str();
        let cmd_two = args[2].as_str();

        match cmd_one {
            "create" => {
                let args_split = args.clone().split_off(3);

                match cmd_two {
                    "database" => {
                        let db = get_flag_val::<String>(&args_split, cli::DB_NAME_FLAG).unwrap();
                        database_manager.create_consumer_database(&db).await;
                    }
                    "user" => {
                        let username =
                            get_flag_val::<String>(&args_split, cli::USERNAME_FLAG).unwrap();
                        let password =
                            get_flag_val::<String>(&args_split, cli::PASSWORD_FLAG).unwrap();
                        database_manager.create_user(username, password).await;
                    }
                    _ => panic!(
                        "Error: Unknown command {}, supported commands are [database, user]",
                        cmd_two
                    ),
                }
            }
            "modify" => match cmd_two {
                "user" => {
                    let cmd_three = args[3].as_str();
                    let args_split = args.clone().split_off(4);

                    match cmd_three {
                        "access" => {
                            let username =
                                get_flag_val::<String>(&args_split, cli::USERNAME_FLAG).unwrap();
                            let database =
                                get_flag_val::<String>(&args_split, cli::DB_NAME_FLAG).unwrap();
                            let access_right =
                                get_flag_val::<u8>(&args_split, cli::ACCESS_RIGHT_FLAG).unwrap();

                            database_manager
                                .modify_user_access(username, database, access_right)
                                .await;
                        }
                        _ => panic!(
                            "Error: Unknown command {}, supported commands are [access]",
                            cmd_three
                        ),
                    }
                }
                _ => panic!(
                    "Error: Unknown command {}, supported commands are [user]",
                    cmd_two
                ),
            },
            _ => panic!(
                "Error: Unknown command {}, supported commands are [create, modify]",
                cmd_one
            ),
        }
    }

    Ok(())
}
