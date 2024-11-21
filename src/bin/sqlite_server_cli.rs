use core::str;
use std::fs::create_dir_all;
use std::path::Path;
use std::{env, iter::Map};

use sha2::{Digest, Sha256};
use sqlite_server::core::{
    db::{execute_query, fetch_all_as_json, AppliedQuery, QueryArg},
    queries,
    state::Usr,
};
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};

include!(concat!(env!("OUT_DIR"), "/gen.rs"));

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
    let root_dir = Path::new(ROOT_DIR);
    let cfg_path = root_dir.join("cfg");
    let consumer_db_path = root_dir.join("db");
    let user_db_full_path_string = format!("{}/{}", cfg_path.to_str().unwrap(), USER_DB_PATH);
    let user_db_full_path = Path::new(&user_db_full_path_string);

    if !user_db_full_path.exists() {
        let _ = create_dir_all(user_db_full_path);
    }

    if !consumer_db_path.exists() {
        let _ = create_dir_all(&consumer_db_path);
    }

    println!("{}", user_db_full_path_string);

    let user_db = format!("{}/{}.db", user_db_full_path_string, USER_DB_PATH);

    if !Sqlite::database_exists(&user_db).await.unwrap_or(false) {
        println!("INIT ");

        match Sqlite::create_database(&user_db).await {
            Ok(_) => {
                println!("CREATE root database DONE");
                let pool = SqlitePool::connect(&format!("sqlite:{}", user_db))
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
                        println!("CREATE users table DONE");

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
                                println!("CREATE users_database_access table DONE");
                            }
                            Err(err) => {
                                let _ = transaction.rollback().await;
                                panic!("CREATE users_database_access table ERROR={}", err);
                            }
                        }
                    }
                    Err(err) => {
                        let _ = transaction.rollback().await;
                        panic!("CREATE users table ERROR={}", err);
                    }
                }
            }
            Err(error) => panic!("CREATE DB ERROR={}", error),
        }
    }

    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        let mut u = "";
        let mut up = "";
        let mut udb = "";
        let mut udba = 0;
        let mut db = "";

        let command = args[1].as_str();
        let sub_command = args[2].as_str();
        for i in 3..args.len() - 1 {
            let flag = args[i].as_str();
            let flag_val = args[i + 1].as_str();

            if command.eq("add") {
                if sub_command.eq("user") {
                    let _ = match flag {
                        "-u" => {
                            if !flag_val.starts_with("-") {
                                u = flag_val;
                            }
                        }
                        "-p" => {
                            if !flag_val.starts_with("-") {
                                up = flag_val;
                            }
                        }
                        "-d" => {
                            if !flag_val.starts_with("-") {
                                udb = flag_val;
                            }
                        }
                        "-a" => {
                            if !flag_val.starts_with("-") {
                                udba = flag_val.parse::<i32>().unwrap_or_default();
                            }
                        }
                        _ => (),
                    };

                    if !u.eq("") && !up.eq("") && !udb.eq("") && udba != 0 {
                        let u_res = base16ct::lower::encode_string(&Sha256::digest(u.as_bytes()));
                        let up_res = base16ct::lower::encode_string(&Sha256::digest(
                            format!("{}{}", u, up).as_bytes(),
                        ));
                        let udb_res =
                            base16ct::lower::encode_string(&Sha256::digest(udb.as_bytes()));

                        println!("{}|{}|{}|{}", u, u_res, up_res, udb_res);

                        let pool = SqlitePool::connect(&format!("sqlite:{}", user_db))
                            .await
                            .unwrap();
                        let mut transaction = pool.begin().await.unwrap();

                        match execute_query(
                            AppliedQuery::new(queries::INSERT_USER).with_args(vec![
                                QueryArg::String(u),
                                QueryArg::String(&u_res),
                                QueryArg::String(&up_res),
                            ]),
                            &mut *transaction,
                        )
                        .await
                        {
                            Ok(_) => {
                                match execute_query(
                                    AppliedQuery::new(queries::INSERT_USER_DATABASE_ACCESS)
                                        .with_args(vec![
                                            QueryArg::String(&udb_res),
                                            QueryArg::Int(udba),
                                            QueryArg::String(&u_res),
                                        ]),
                                    &mut *transaction,
                                )
                                .await
                                {
                                    Ok(_) => {
                                        let _ = transaction.commit().await;
                                        println!("INSERT INTO users_database_access OK");
                                    }
                                    Err(err) => {
                                        let _ = transaction.rollback().await;
                                        panic!("ERROR={}", err);
                                    }
                                }
                                println!("INSERT INTO users OK");
                            }
                            Err(err) => {
                                let _ = transaction.rollback().await;
                                panic!("ERROR={}", err);
                            }
                        };
                    }
                }
            } else if command.eq("create") {
                if sub_command.eq("database") {
                    let _ = match flag {
                        "-db" => {
                            if !flag_val.starts_with("-") {
                                db = flag_val;
                            }
                        }
                        _ => (),
                    };

                    if !db.eq("") {
                        let db_name: String = db
                            .chars()
                            .map(|x| match x {
                                // '-' => '_',
                                '!'..='/' => '\0',
                                ':'..='@' => '\0',
                                _ => x,
                            })
                            .collect();
                        println!("{}: {:?}", db_name.len(), db_name);

                        let trimmed_db_name = db_name.replace("..", "").replace('\0', "");
                        println!("{}: {:?}", trimmed_db_name.len(), trimmed_db_name);
                        let db_name_hash = base16ct::lower::encode_string(&Sha256::digest(
                            trimmed_db_name.as_bytes(),
                        ));

                        let consumer_db_full_path_string =
                            format!("{}/{}", consumer_db_path.to_str().unwrap(), db_name_hash);
                        let consumer_db_full_path = Path::new(&consumer_db_full_path_string);
                        println!("after format {}", consumer_db_full_path_string);

                        if !consumer_db_full_path.exists() {
                            let _ = create_dir_all(&consumer_db_full_path);
                        }

                        let consumer_db =
                            format!("{}/{}.db", consumer_db_full_path_string, db_name_hash);

                        if !Sqlite::database_exists(&consumer_db).await.unwrap_or(false) {
                            println!("Creating database {}", &consumer_db);

                            match Sqlite::create_database(&consumer_db).await {
                                Ok(_) => println!("Create db success"),
                                Err(error) => panic!("error: {}", error),
                            }
                        } else {
                            println!("Database already exists");
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
