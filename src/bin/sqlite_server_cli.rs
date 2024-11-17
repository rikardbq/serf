use core::str;
use std::env;
use std::fs::create_dir_all;
use std::path::Path;

use sha2::{Digest, Sha256};
use sqlite_server::core::db::{execute_query, fetch_all_as_json, AppliedQuery, QueryArg};
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};

include!(concat!(env!("OUT_DIR"), "/gen.rs"));

/*
ToDo:
    migrate the database and tables creation
        + initial user create to build.rs


*/

#[tokio::main]
async fn main() -> std::io::Result<()> {
    println!("IM THE CLI !");
    println!("{}", message());

    let path_hash = base16ct::lower::encode_string(&Sha256::digest("cfg_root_db_path".as_bytes()));
    let root_db_path_string = format!("./config_/{}", path_hash);
    let root_db_path = Path::new(&root_db_path_string);
    if !root_db_path.exists() {
        let _ = create_dir_all(root_db_path);
    }
    let root_db = format!("{}/{}.db", &root_db_path_string, path_hash);
    if !Sqlite::database_exists(&root_db).await.unwrap_or(false) {
        println!("INIT ");

        match Sqlite::create_database(&root_db).await {
            Ok(_) => {
                println!("CREATE root database DONE");
                let pool = SqlitePool::connect(&format!("sqlite:{}", root_db))
                    .await
                    .unwrap();

                // CREATE users
                match execute_query(
                    AppliedQuery::new(
                        r#"
                        CREATE TABLE IF NOT EXISTS users (
                            id INTEGER PRIMARY KEY NOT NULL, 
                            username TEXT NOT NULL UNIQUE, 
                            username_hash TEXT NOT NULL, 
                            username_password_hash TEXT NOT NULL
                        );
                        "#,
                    ),
                    &pool,
                )
                .await
                {
                    Ok(_) => {
                        println!("CREATE users table DONE");

                        // CREATE database accesses
                        // access_right can contain, or at least handle 1, 2 or 3 as value
                        // all other values will be seen as non-functioning
                        match execute_query(
                            AppliedQuery::new(
                                r#"
                                CREATE TABLE IF NOT EXISTS users_database_access (
                                    id INTEGER PRIMARY KEY NOT NULL,
                                    database TEXT NOT NULL,
                                    access_right INTEGER NOT NULL DEFAULT 1,
                                    user INTEGER NOT NULL,
                                    FOREIGN KEY (user)
                                    REFERENCES users (username_hash) 
                                        ON UPDATE CASCADE
                                        ON DELETE CASCADE
                                );
                                "#,
                            ),
                            &pool,
                        )
                        .await
                        {
                            Ok(_) => println!("CREATE users_database_access table DONE"),
                            Err(err) => panic!("CREATE users_database_access table ERROR={}", err),
                        }
                    }
                    Err(err) => panic!("CREATE users table ERROR={}", err),
                }
            }
            Err(error) => panic!("CREATE DB ERROR={}", error),
        }
    }

    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        let mut u = "";
        let mut p = "";
        let mut db = "";

        let command = args[1].as_str();
        let sub_command = args[2].as_str();
        for i in 3..args.len() - 1 {
            let flag = args[i].as_str();
            let flag_val = args[i + 1].as_str();
            println!("FLAG= {}\nFLAG_VAL= {}", flag, flag_val);
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
                                p = flag_val;
                            }
                        }
                        _ => (),
                    };

                    if !u.eq("") && !p.eq("") {
                        let u_hash = Sha256::digest(u.as_bytes());
                        let up_hash = Sha256::digest(format!("{}{}", u, p).as_bytes());
                        let u_res = base16ct::lower::encode_string(&u_hash);
                        let up_res = base16ct::lower::encode_string(&up_hash);

                        println!("{}|{:?}|{:?}", u, u_res, up_res);

                        let pool = SqlitePool::connect(&format!("sqlite:{}", root_db))
                            .await
                            .unwrap();

                        match execute_query(
                            AppliedQuery::new(
                                r#"
                                INSERT OR IGNORE INTO users(
                                    username,
                                    username_hash,
                                    username_password_hash
                                ) VALUES(?, ?, ?);
                                "#,
                            )
                            .with_args(vec![
                                QueryArg::String(u),
                                QueryArg::String(&u_res),
                                QueryArg::String(&up_res),
                            ]),
                            &pool,
                        )
                        .await
                        {
                            Ok(_) => println!("INSERT OK"),
                            Err(err) => panic!("ERROR={}", err),
                        };
                    }
                }
            } else if flag.eq("create") {
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

                        let db_url = format!("sqlite:./config_/{}.db", trimmed_db_name);
                        println!("after format {}", db_url);

                        if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
                            println!("Creating database {}", &db_url);

                            match Sqlite::create_database(&db_url).await {
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
