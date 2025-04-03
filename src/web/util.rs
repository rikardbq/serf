use actix_web::{http::header::HeaderValue, HttpResponse, HttpResponseBuilder};
use sqlx::SqlitePool;

use crate::core::{
    db::{execute_query, fetch_all_as_json, AppliedQuery},
    error::{
        HeaderMalformedError, HeaderMissingError, SerfError, UndefinedError, UserNotAllowedError,
    },
    serf_proto::{claims::Dat, Error, FetchResponse, MutationResponse, QueryRequest, Sub},
};

use super::proto::{encode_proto, ProtoPackage};

async fn handle_mutate<'a>(
    request_query: &'a QueryRequest,
    user_access: u8,
    username_password_hash: &'a str,
    db: &'a SqlitePool,
) -> Result<ProtoPackage, Error> {
    if user_access >= 2 {
        let mut transaction = db.begin().await.unwrap();
        match execute_query(
            AppliedQuery::new(&request_query.query).with_args(&request_query.parts),
            &mut *transaction,
        )
        .await
        {
            Ok(res) => {
                let _ = &mut transaction.commit().await;
                encode_proto(
                    MutationResponse::as_dat(res.rows_affected(), res.last_insert_rowid() as u64),
                    Sub::Data,
                    username_password_hash,
                )
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
    request_query: &'a QueryRequest,
    user_access: u8,
    username_password_hash: &'a str,
    db: &'a SqlitePool,
) -> Result<ProtoPackage, Error> {
    if user_access >= 1 {
        match fetch_all_as_json(
            AppliedQuery::new(&request_query.query).with_args(&request_query.parts),
            &db,
        )
        .await
        {
            Ok(res) => encode_proto(
                FetchResponse::as_dat(serde_json::to_vec(&res).unwrap()),
                Sub::Data,
                username_password_hash,
            ),
            Err(e) => Err(UndefinedError::with_message(
                e.as_database_error().unwrap().message(),
            )),
        }
    } else {
        Err(UserNotAllowedError::default())
    }
}

pub async fn get_query_result_proto_package<'a>(
    dat: &'a Option<Dat>,
    sub: Sub,
    user_access: u8,
    username_password_hash: &'a str,
    db: &'a SqlitePool,
) -> Result<ProtoPackage, Error> {
    if let Some(Dat::QueryRequest(dat)) = dat {
        match sub {
            Sub::Mutate => handle_mutate(dat, user_access, username_password_hash, &db).await,
            Sub::Fetch => handle_fetch(dat, user_access, username_password_hash, &db).await,
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

pub fn build_proto_response(
    response_builder: &mut HttpResponseBuilder,
    proto_package: ProtoPackage,
) -> HttpResponse {
    response_builder
        .insert_header(("Content-Type", "application/protobuf"))
        .insert_header(("0", proto_package.signature))
        .body(proto_package.data)
}
