use core::str;
use std::sync::Arc;
use std::time::Duration;

use actix_web::{http::header::HeaderValue, web};
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;

use crate::web::jwt::{generate_claims, Claims, RequestQuery, Sub};

use super::{
    constants::errors::{self, ErrorReason},
    db::{execute_query, fetch_all_as_json, AppliedQuery},
    state::{AppState, User},
};

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
    db_name: &'a str,
) -> Result<(User, u8), &'a str> {
    let users_clone: Arc<papaya::HashMap<String, User>> = Arc::clone(&data.users);
    let users = users_clone.pin();

    match users.get(header_u_) {
        Some(u) => {
            let db_access_rights_pin = u.db_access_rights.pin();
            match db_access_rights_pin.get(db_name) {
                Some(ar) => Ok((u.to_owned(), *ar)),
                None => Err(errors::ERROR_USER_NO_DATABASE_ACCESS),
            }
        }
        None => Err(errors::ERROR_UNKNOWN_USER),
    }
}

pub async fn get_db_connections<'a>(
    data: &'a web::Data<AppState>,
    db_name: &'a str,
) -> Result<SqlitePool, &'a str> {
    let db_connections_clone: Arc<papaya::HashMap<String, SqlitePool>> =
        Arc::clone(&data.db_connections);
    let db_connections = db_connections_clone.pin();

    if !db_connections.contains_key(db_name) {
        println!(
            "Database connection is not opened, trying to open database {}",
            db_name
        );
        if let Ok(pool) = SqlitePoolOptions::new()
            .max_connections(data.db_max_connections)
            .idle_timeout(Duration::from_secs(data.db_max_idle_time))
            .connect(&format!(
                "sqlite:{}/{}/{}.db",
                data.db_path, db_name, db_name
            ))
            .await
        {
            db_connections.insert(db_name.to_owned(), pool);
        } else {
            return Err(errors::ERROR_DATABASE_NOT_FOUND);
        }
    }

    Ok(db_connections.get(db_name).unwrap().to_owned())
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

pub fn get_header_value(header: Option<&HeaderValue>) -> Result<&str, &str> {
    let header_value = match header {
        Some(hdr) => {
            match hdr.to_str() {
                Ok(hdr_val) => hdr_val,
                Err(_) => errors::ERROR_MALFORMED_HEADER
            }
        },
        None => errors::ERROR_MISSING_HEADER,
    };

    Ok(header_value)
}
