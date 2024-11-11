use core::str;
use std::env;

use std::fs::OpenOptions;
use std::io::prelude::*;

use sha2::{Digest, Sha256};
use sqlx::{migrate::MigrateDatabase, Sqlite};

pub const USR_CONFIG_LOCATION: &str = "./config_/usr_";

#[tokio::main]
async fn main() -> std::io::Result<()> {
    println!("IM THE CLI !");
    let args: Vec<String> = env::args().collect();

    let mut u = "";
    let mut p = "";
    let mut db = "";

    if args.len() > 1 {
        let action = args[1].as_str();
        for i in 1..args.len() {
            let _ = match &args[i] as &str {
                "-db" => {
                    if !args[i + 1].starts_with("-") {
                        db = &args[i + 1];
                    }
                }
                "-u" => {
                    if !args[i + 1].starts_with("-") {
                        u = &args[i + 1];
                    }
                }
                "-p" => {
                    if !args[i + 1].starts_with("-") {
                        p = &args[i + 1];
                    }
                }
                _ => (),
            };
        }

        let u_hash = Sha256::digest(u.as_bytes());
        let up_hash = Sha256::digest(format!("{}{}", u, p).as_bytes());
        let u_res = base16ct::lower::encode_string(&u_hash);
        let up_res = base16ct::lower::encode_string(&up_hash);
        println!("{}|{:?}|{:?}|", u, u_res, up_res);

        if action.eq("add") {
            let mut file = OpenOptions::new()
                .write(true)
                .append(true)
                .open(USR_CONFIG_LOCATION)
                .unwrap();

            // if let Err(e) = writeln!(file, "A new line!") {
            //     eprintln!("Couldn't write to file: {}", e);
            // }

            file.write_all(format!("\n{}|{}|{}|", u, u_res, up_res).as_bytes())
                .unwrap();
        }
        if action.eq("create") {
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
                println!("after format {}", db_url); // Xxxxx, xxxxx?
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
    Ok(())
}
