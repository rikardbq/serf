use actix_web::{http::header::HeaderValue, HttpRequest, HttpResponse, HttpResponseBuilder};
use sqlx::SqlitePool;

use crate::core::{
    constants::queries,
    db::{execute_query, fetch_all_as_json, AppliedQuery},
    error::{
        HeaderMalformedError, HeaderMissingError, SerfError, UndefinedError, UserNotAllowedError,
    },
    serf_proto::{
        claims::Dat, query_arg, Error, FetchResponse, MigrationRequest, MigrationResponse,
        MutationResponse, QueryArg, QueryRequest, Sub,
    },
};

use super::proto::{encode_proto, ProtoPackage};

async fn handle_migrate<'a>(
    migration: &'a MigrationRequest,
    user_access: u8,
    username_password_hash: &'a str,
    db: &'a SqlitePool,
) -> Result<ProtoPackage, Error> {
    if user_access >= 2 {
        let mut transaction = db.begin().await.unwrap();

        // create if not exist, will enter Ok clause even if it exists
        match execute_query(
            AppliedQuery::new(queries::CREATE_MIGRATIONS_TABLE),
            &mut *transaction,
        )
        .await
        {
            Ok(_) => {
                if let Err(e) = execute_query(
                    AppliedQuery::new(queries::INSERT_MIGRATION).with_args(&vec![
                        QueryArg::new(query_arg::Value::String(migration.name.clone())),
                        QueryArg::new(query_arg::Value::String(migration.query.clone())),
                    ]),
                    &mut *transaction,
                )
                .await
                {
                    let _ = &mut transaction.rollback().await;
                    panic!("{e}");
                }
            }
            Err(e) => {
                let _ = &mut transaction.rollback().await;
                panic!("{e}");
            }
        };

        match execute_query(AppliedQuery::new(&migration.query), &mut *transaction).await {
            Ok(_) => {
                let _ = transaction.commit().await;
                encode_proto(
                    MigrationResponse::as_dat(true),
                    Sub::Data,
                    username_password_hash,
                )
            }
            Err(e) => {
                let _ = transaction.rollback().await;
                eprintln!("{e}");
                encode_proto(
                    MigrationResponse::as_dat(false),
                    Sub::Data,
                    username_password_hash,
                )
            }
        }
    } else {
        Err(UserNotAllowedError::default())
    }
}

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

pub async fn get_proto_package_result<'a>(
    dat: &'a Option<Dat>,
    sub: Sub,
    user_access: u8,
    username_password_hash: &'a str,
    db: &'a SqlitePool,
) -> Result<ProtoPackage, Error> {
    match dat {
        Some(Dat::MigrationRequest(dat)) => match sub {
            Sub::Migrate => handle_migrate(dat, user_access, username_password_hash, &db).await,
            _ => Err(UndefinedError::default()),
        },
        Some(Dat::QueryRequest(dat)) => match sub {
            Sub::Mutate => handle_mutate(dat, user_access, username_password_hash, &db).await,
            Sub::Fetch => handle_fetch(dat, user_access, username_password_hash, &db).await,
            _ => Err(UndefinedError::default()),
        },
        _ => Err(UndefinedError::default()),
    }
}

fn get_header_value(header: Option<&HeaderValue>) -> Result<&str, Error> {
    match header {
        Some(hdr) => match hdr.to_str() {
            Ok(hdr_val) => Ok(hdr_val),
            Err(_) => Err(HeaderMalformedError::default()),
        },
        None => Err(HeaderMissingError::default()),
    }
}

pub fn extract_headers<'a>(req: &'a HttpRequest) -> Result<(&'a str, &'a str), Error> {
    let header_username_hash = get_header_value(req.headers().get("0"))?;
    let header_proto_signature = get_header_value(req.headers().get("1"))?;

    Ok((header_username_hash, header_proto_signature))
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
