use core::str;
use std::sync::Arc;
use std::time::Duration;

use actix_web::web;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;

use crate::web::jwt::{generate_claims, Claims, RequestQuery, Sub};

use super::{
    constants::errors::{self, ErrorReason},
    db::{execute_query, fetch_all_as_json, AppliedQuery},
    state::{AppState, Usr},
};

// pub const USR_CONFIG_LOCATION: &str = "./config_/usr_";

// pub fn usr_config_buffer(location: &str) -> Vec<u8> {
//     let mut buffer = Vec::<u8>::new();

//     let _ = (match File::open(location) {
//         Ok(f) => {
//             println!("Using usr config from {}", location);
//             f
//         }
//         Err(err) => {
//             eprintln!("Error reading usr config with ERROR={}", err);
//             panic!()
//         }
//     })
//     .read_to_end(&mut buffer);

//     buffer
// }

// pub fn parse_usr_config(config_buffer: Vec<u8>) -> papaya::HashMap<String, Usr> {
//     let usr_config = str::from_utf8(&config_buffer).unwrap();
//     let usr_entries: Vec<&str> = usr_config.split("\n").collect();
//     let usr_map = papaya::HashMap::new();
//     let usr_map_ref = usr_map.pin();

//     usr_entries.iter().for_each(move |x| {
//         let t: Vec<&str> = x.split("|").collect();
//         let access_entries: Vec<&str> = t[3].split(",").collect();
//         let mut access_map = std::collections::HashMap::new();

//         access_entries.iter().for_each(|y| {
//             let u: Vec<&str> = y.split(":").collect();
//             let _ = access_map.insert(String::from(u[0]), u[1].parse::<u8>().unwrap());
//         });

//         let _ = usr_map_ref.insert(
//             String::from(t[1]),
//             Usr {
//                 u: String::from(t[0]),
//                 up_hash: String::from(t[2]),
//                 db_ar: access_map,
//             },
//         );
//     });

//     usr_map
// }

/*

    // setup user access to be checked later on whenever a client tries to read or write to a given DB
    // re-use setup as part of usr_ conf file update to avoid downtime whenever new entries in access file needs to be handled
    let usr_config = parse_usr_config(USR_CONFIG_LOCATION);
    let usr_entries: Vec<&str> = usr_config.as_str().split("\n").collect();
    let mut usr_map = HashMap::new();

    usr_entries.iter().for_each(|x| {
        let t: Vec<&str> = x.split("|").collect();
        let access_entries: Vec<&str> = t[3].split(",").collect();
        let mut access_map = HashMap::new();

        access_entries.iter().for_each(|y| {
            let u: Vec<&str> = y.split(":").collect();
            let _ = &mut access_map.insert(String::from(u[0]), u[1].parse::<u8>().unwrap());
        });

        let _ = &mut usr_map.insert(String::from(t[1]), Usr {
            u: String::from(t[0]),
            up_hash: String::from(t[2]),
            db_ar: access_map,
        });
    });

    ---

    simply call the usr_ db "./<project_root>/r_/usr_.db"
    anything else lives in a different folder alltogether "./<project_root>/p_/<whatever>.db"

*/

pub struct Error<'a> {
    pub message: &'a str,
    pub reason: Option<ErrorReason>,
}

impl<'a> Error<'a> {
    pub fn new(message: &'a str) -> Self {
        Error {
            message: message,
            reason: None,
        }
    }

    pub fn with_reason(self, reason: ErrorReason) -> Self {
        Error {
            reason: Some(reason),
            ..self
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

pub fn get_user_entry_and_access<'a>(
    data: &'a web::Data<AppState>,
    header_u_: &'a str,
    database_name: &'a String,
) -> Result<(Usr, u8), &'a str> {
    let usr_clone: Arc<papaya::HashMap<String, Usr>> = Arc::clone(&data.usr);
    let usr = usr_clone.pin();

    match usr.get(header_u_) {
        Some(u) => match u.db_ar.get(database_name) {
            Some(ar) => Ok((u.to_owned(), *ar)),
            None => Err(errors::ERROR_USER_NO_DATABASE_ACCESS),
        },
        None => Err(errors::ERROR_UNKNOWN_USER),
    }
}

pub async fn get_database_connections<'a>(
    data: &'a web::Data<AppState>,
    database_name: &'a String,
) -> Result<SqlitePool, &'a str> {
    let database_connections_clone: Arc<papaya::HashMap<String, SqlitePool>> =
        Arc::clone(&data.database_connections);
    let database_connections = database_connections_clone.pin();

    if !database_connections.contains_key(database_name) {
        println!(
            "database connection is not opened, trying to open database {}",
            database_name
        );
        if let Ok(pool) = SqlitePoolOptions::new()
            .max_connections(data.db_max_conn)
            .idle_timeout(Duration::from_secs(data.db_max_idle_time))
            .connect(&format!(
                "sqlite:{}/{}/{}.db",
                data.db_path, database_name, database_name
            ))
            .await
        {
            database_connections.insert(database_name.to_owned(), pool);
        } else {
            return Err(errors::ERROR_DATABASE_NOT_FOUND);
        }
    }

    Ok(database_connections.get(database_name).unwrap().to_owned())
}

pub async fn get_query_result_claims<'a>(
    query_claims: Claims,
    user_access: u8,
    db: &'a SqlitePool,
) -> Result<Claims, Error<'a>> {
    let dat: RequestQuery = serde_json::from_str(&query_claims.dat).unwrap();
    let response_claims_result = match query_claims.sub {
        Sub::MUTATE => {
            if user_access >= 2 {
                let mut transaction = db.begin().await.unwrap();

                match execute_query(
                    AppliedQuery::new(&dat.query).with_args(dat.parts),
                    &mut *transaction,
                )
                .await
                {
                    Ok(res) => {
                        let _ = &mut transaction.commit().await;

                        Ok(generate_claims(
                            serde_json::to_string(&res.rows_affected()).unwrap(),
                            Sub::DATA,
                        ))
                    }
                    Err(_) => {
                        let _ = &mut transaction.rollback().await;

                        Err(Error::new(errors::ERROR_UNSPECIFIED))
                    }
                }
            } else {
                Err(Error::new(errors::ERROR_FORBIDDEN).with_reason(ErrorReason::UserNotAllowed))
            }
        }
        Sub::FETCH => {
            if user_access >= 1 {
                let result =
                    fetch_all_as_json(AppliedQuery::new(&dat.query).with_args(dat.parts), &db)
                        .await
                        .unwrap();

                Ok(generate_claims(
                    serde_json::to_string(&result).unwrap(),
                    Sub::DATA,
                ))
            } else {
                Err(Error::new(errors::ERROR_FORBIDDEN).with_reason(ErrorReason::UserNotAllowed))
            }
        }
        _ => Err(Error::new(errors::ERROR_NOT_ACCEPTABLE).with_reason(ErrorReason::InvalidSubject)),
    };

    response_claims_result
}
