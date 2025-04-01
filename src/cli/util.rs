use std::{
    fs,
    path::{Path, PathBuf},
};

use regex::Regex;
use sha2::{Digest, Sha256};
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};

use crate::core::{
    constants::queries,
    db::{execute_query, AppliedQuery},
    serf_proto::{QueryArg, query_arg}
};

include!(concat!(env!("OUT_DIR"), "/gen.rs"));

pub struct DatabaseManager {
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
            consumer_db_base_path,
            user_db_base_path,
            user_db_full_path_string,
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
                let pool =
                    SqlitePool::connect(&format!("sqlite:{}", self.user_db_full_path_string))
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

    pub async fn create_database(&self, db_name: &str) {
        if !db_name.eq("") {
            let regex = Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap();
            if !regex.is_match(db_name) {
                panic!("Error: Database name format must follow either one or a combination of the patterns [a-z, A-Z, 0-9, _, -]");
            }

            let db_name_hash = base16ct::lower::encode_string(&Sha256::digest(db_name.as_bytes()));
            let consumer_db_full_path_string = format!(
                "{}/{}",
                self.consumer_db_base_path.to_str().unwrap(),
                db_name_hash
            );
            let consumer_db_full_path = Path::new(&consumer_db_full_path_string);
            let consumer_db = format!("{}/{}.db", consumer_db_full_path_string, db_name_hash);

            if !consumer_db_full_path.exists() {
                let _ = fs::create_dir_all(&consumer_db_full_path);
            }

            if !Sqlite::database_exists(&consumer_db).await.unwrap_or(false) {
                match Sqlite::create_database(&consumer_db).await {
                    Ok(_) => {
                        println!("Successfully created db {} as {}", db_name, db_name_hash);
                        let _ = fs::write(
                            format!("{}/{}", consumer_db_full_path_string, db_name),
                            db_name_hash,
                        );
                    }
                    Err(err) => panic!("Error: {}", err),
                }
            } else {
                panic!("Error: Database already exists");
            }
        }
    }

    pub async fn create_user(&self, username: String, password: String) {
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
                AppliedQuery::new(queries::INSERT_USER).with_args(&vec![
                    QueryArg::new(query_arg::Value::String(username)),
                    QueryArg::new(query_arg::Value::String(username_hash)),
                    QueryArg::new(query_arg::Value::String(username_password_hash)),
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
            panic!("Error: Must provide username and password with flags [-u, -p]");
        }
    }

    pub async fn modify_user_access(
        &self,
        username: String,
        database_name: String,
        access_right: u8,
    ) {
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
                AppliedQuery::new(queries::UPSERT_USER_DATABASE_ACCESS).with_args(&vec![
                   QueryArg::new(query_arg::Value::String(database_name)),
                   QueryArg::new(query_arg::Value::String(database_name_hash)),
                   QueryArg::new(query_arg::Value::Int(access_right as i64)),
                   QueryArg::new(query_arg::Value::String(username_hash)),
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
            panic!("Error: Must provide username, database and access right(1-3) with flags [-u, -db, -a]");
        }
    }
}

pub fn get_flag_val<'a, T>(args: &'a Vec<String>, flag: &'a str) -> Option<T>
where
    T: std::str::FromStr,
{
    let mut res = None;

    for i in 0..args.len() - 1 {
        let args_flag = &args[i];
        let args_flag_val = &args[i + 1];

        if args_flag == flag {
            if !args_flag_val.starts_with("-") {
                if let Ok(parsed_val) = args_flag_val.parse::<T>() {
                    res = Some(parsed_val);
                    break;
                } else {
                    panic!("Flag value cannot be parsed");
                }
            }
        };
    }

    res
}
