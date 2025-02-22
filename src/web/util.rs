use actix_web::http::header::HeaderValue;
use sqlx::SqlitePool;

use crate::core::{
    db::{execute_query, fetch_all_as_json, AppliedQuery},
    error::{
        Error, HeaderMalformedError, HeaderMissingError, SerfError, UndefinedError,
        UserNotAllowedError,
    },
};

use super::jwt::{generate_claims, Claims, DatKind, MutationResponse, QueryRequest, Sub};

async fn handle_mutate<'a>(
    request_query: QueryRequest,
    user_access: u8,
    db: &'a SqlitePool,
) -> Result<Claims, Error> {
    if user_access >= 2 {
        let mut transaction = db.begin().await.unwrap();
        match execute_query(
            AppliedQuery::new(&request_query.query).with_args(request_query.parts),
            &mut *transaction,
        )
        .await
        {
            Ok(res) => {
                let _ = &mut transaction.commit().await;
                Ok(generate_claims(
                    DatKind::MutationRes(MutationResponse {
                        rows_affected: res.rows_affected(),
                        last_insert_rowid: res.last_insert_rowid(),
                    }),
                    Sub::DATA,
                ))
            }
            Err(e) => {
                let _ = &mut transaction.rollback().await;
                Err(UndefinedError::with_message(
                    e.as_database_error().unwrap().message(),
                ))
            }
        }
    } else {
        Err(UserNotAllowedError::default())
    }
}

async fn handle_fetch<'a>(
    request_query: QueryRequest,
    user_access: u8,
    db: &'a SqlitePool,
) -> Result<Claims, Error> {
    if user_access >= 1 {
        match fetch_all_as_json(
            AppliedQuery::new(&request_query.query).with_args(request_query.parts),
            &db,
        )
        .await
        {
            Ok(res) => Ok(generate_claims(DatKind::FetchRes(res), Sub::DATA)),
            Err(e) => Err(UndefinedError::with_message(
                e.as_database_error().unwrap().message(),
            )),
        }
    } else {
        Err(UserNotAllowedError::default())
    }
}

pub async fn get_query_result_claims<'a>(
    query_claims: Claims,
    user_access: u8,
    db: &'a SqlitePool,
) -> Result<Claims, Error> {
    if let DatKind::QueryReq(dat) = query_claims.dat {
        match query_claims.sub {
            Sub::MUTATE => handle_mutate(dat, user_access, &db).await,
            Sub::FETCH => handle_fetch(dat, user_access, &db).await,
            _ => Err(UndefinedError::default()),
        }
    } else {
        Err(UndefinedError::default())
    }
}

pub fn get_header_value(header: Option<&HeaderValue>) -> Result<&str, Error> {
    match header {
        Some(hdr) => match hdr.to_str() {
            Ok(hdr_val) => Ok(hdr_val),
            Err(_) => Err(HeaderMalformedError::default()),
        },
        None => Err(HeaderMissingError::default()),
    }
}
