use core::str;
use std::fs;
use std::path::Path;
use std::{env, path::PathBuf};

use sha2::{Digest, Sha256};
use sqlite_server::core::{
    constants::queries,
    db::{execute_query, AppliedQuery, QueryArg},
    util::get_flag_val,
};
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};

include!(concat!(env!("OUT_DIR"), "/gen.rs"));

struct DatabaseManager {
    pub consumer_db_base_path: PathBuf,
    pub user_db_base_path: PathBuf,
    pub user_db_full_path_string: String,
}

impl DatabaseManager {
    pub fn new() -> DatabaseManager {
        let root_dir = Path::new(ROOT_DIR);
        let cfg_path = root_dir.join("cfg");
        let consumer_db_base_path = root_dir.join("db");
        let user_db_base_path = cfg_path.join(USER_DB_HASH);
        let user_db_full_path_string = format!(
            "{}/{}.db",
            user_db_base_path.to_str().unwrap(),
            USER_DB_HASH
        );

        DatabaseManager {
            consumer_db_base_path: consumer_db_base_path,
            user_db_base_path: user_db_base_path,
            user_db_full_path_string: user_db_full_path_string
        }
    }

    pub async fn init(&self) {
        if !self.user_db_base_path.exists() {
            let _ = fs::create_dir_all(&self.user_db_base_path);
        }

        if !self.consumer_db_base_path.exists() {
            let _ = fs::create_dir_all(&self.consumer_db_base_path);
        }

        match Sqlite::create_database(&self.user_db_full_path_string).await {
            Ok(_) => {
                let pool = SqlitePool::connect(&format!("sqlite:{}", self.user_db_full_path_string))
                    .await
                    .unwrap();
                let mut transaction = pool.begin().await.unwrap();

                // CREATE users
                match execute_query(
                    AppliedQuery::new(queries::CREATE_USERS_TABLE),
                    &mut *transaction,
                )
                .await
                {
                    Ok(_) => {
                        // CREATE database accesses
                        // access_right can contain, or at least handle 1, 2 or 3 as value
                        // all other values will be seen as non-functioning
                        match execute_query(
                            AppliedQuery::new(queries::CREATE_USERS_DATABASE_ACCESS_TABLE),
                            &mut *transaction,
                        )
                        .await
                        {
                            Ok(_) => {
                                let _ = transaction.commit().await;
                            }
                            Err(err) => {
                                let _ = transaction.rollback().await;
                                panic!("Error: {}", err);
                            }
                        }
                    }
                    Err(err) => {
                        let _ = transaction.rollback().await;
                        panic!("Error: {}", err);
                    }
                }
            }
            Err(err) => panic!("Error: {}", err),
        }
    }

    pub async fn create_database(&self, database_name: &str) {
        if !database_name.eq("") {
            let db_name: String = database_name
                .chars()
                .map(|x| match x {
                    // '-' => '_',
                    '!'..='/' => '\0',
                    ':'..='@' => '\0',
                    _ => x,
                })
                .collect();
            let trimmed_db_name = db_name.replace("..", "").replace('\0', "");
            let db_name_hash =
                base16ct::lower::encode_string(&Sha256::digest(trimmed_db_name.as_bytes()));

            let consumer_db_full_path_string = format!(
                "{}/{}",
                self.consumer_db_base_path.to_str().unwrap(),
                db_name_hash
            );
            let consumer_db_full_path = Path::new(&consumer_db_full_path_string);

            if !consumer_db_full_path.exists() {
                let _ = fs::create_dir_all(&consumer_db_full_path);
            }

            let consumer_db = format!("{}/{}.db", consumer_db_full_path_string, db_name_hash);

            if !Sqlite::database_exists(&consumer_db).await.unwrap_or(false) {
                match Sqlite::create_database(&consumer_db).await {
                    Ok(_) => {
                        println!(
                            "Successfully created db {} as {}",
                            trimmed_db_name, db_name_hash
                        );
                        let _ = fs::write(
                            format!("{}/{}", consumer_db_full_path_string, trimmed_db_name),
                            db_name_hash,
                        );
                    }
                    Err(error) => panic!("Error: {}", error),
                }
            } else {
                println!("Database already exists");
            }
        }
    }

    pub async fn create_user(&self, username: &str, password: &str) {
        if !username.eq("") && !password.eq("") {
            let username_hash =
                base16ct::lower::encode_string(&Sha256::digest(username.as_bytes()));
            let username_password_hash = base16ct::lower::encode_string(&Sha256::digest(
                format!("{}{}", username, password).as_bytes(),
            ));

            let pool = SqlitePool::connect(&format!("sqlite:{}", self.user_db_full_path_string))
                .await
                .unwrap();
            let mut transaction = pool.begin().await.unwrap();

            let _ = match execute_query(
                AppliedQuery::new(queries::INSERT_USER).with_args(vec![
                    QueryArg::String(username),
                    QueryArg::String(&username_hash),
                    QueryArg::String(&username_password_hash),
                ]),
                &mut *transaction,
            )
            .await
            {
                Ok(_) => transaction.commit().await,
                Err(err) => {
                    let _ = transaction.rollback().await;
                    panic!("Error: {}", err);
                }
            };
        } else {
            panic!("Error: Must provide username and password");
        }
    }

    pub async fn modify_user_access(&self, username: &str, database_name: &str, access_right: i32) {
        if !username.eq("") && !database_name.eq("") && access_right != 0 {
            let username_hash =
                base16ct::lower::encode_string(&Sha256::digest(username.as_bytes()));
            let database_name_hash =
                base16ct::lower::encode_string(&Sha256::digest(database_name.as_bytes()));

            let pool = SqlitePool::connect(&format!("sqlite:{}", self.user_db_full_path_string))
                .await
                .unwrap();
            let mut transaction = pool.begin().await.unwrap();

            let _ = match execute_query(
                AppliedQuery::new(queries::UPSERT_USER_DATABASE_ACCESS).with_args(vec![
                    QueryArg::String(database_name),
                    QueryArg::String(&database_name_hash),
                    QueryArg::Int(access_right),
                    QueryArg::String(&username_hash),
                ]),
                &mut *transaction,
            )
            .await
            {
                Ok(_) => {
                    let _ = transaction.commit().await;
                }
                Err(err) => {
                    let _ = transaction.rollback().await;
                    panic!("Error: {}", err);
                }
            };
        } else {
            panic!("Error: Must provide username, database and access right(1-3)");
        }
    }
}

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
        database_manager.init().await
    }

    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let cmd_one = args[1].as_str();
        let cmd_two = args[2].as_str();

        if cmd_one.eq("create") {
            let args_split = args.clone().split_off(3);
            if cmd_two.eq("database") {
                let db = get_flag_val::<String>(&args_split, "-db").unwrap();
                database_manager.create_database(&db).await;
            }

            if cmd_two.eq("user") {
                let username = get_flag_val::<String>(&args_split, "-u").unwrap();
                let password = get_flag_val::<String>(&args_split, "-p").unwrap();

                database_manager.create_user(&username, &password).await;
            }
        } else if cmd_one.eq("modify") {
            if cmd_two.eq("user") {
                let cmd_three = args[3].as_str();
                let args_split = args.clone().split_off(4);
                if cmd_three.eq("access") {
                    let username = get_flag_val::<String>(&args_split, "-u").unwrap();
                    let database = get_flag_val::<String>(&args_split, "-db").unwrap();
                    let access_right = get_flag_val::<i32>(&args_split, "-a").unwrap();

                    database_manager
                        .modify_user_access(&username, &database, access_right)
                        .await;
                }
            }
        }
    }

    Ok(())
}
