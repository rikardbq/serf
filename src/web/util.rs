use std::future::Future;

use actix_web::{http::header::HeaderValue, HttpRequest, HttpResponse, HttpResponseBuilder};
use sqlx::SqlitePool;

#[cfg(test)]
use mockall::automock;

use crate::core::{
    constants::queries,
    db::{execute_query, fetch_all_as_json, AppliedQuery},
    error::{
        DatabaseError, HeaderMalformedError, HeaderMissingError, SerfError, UndefinedError,
        UserNotAllowedError,
    },
    serf_proto::{
        claims::Dat, query_arg, Claims, Error, FetchResponse, MigrationRequest, MigrationResponse,
        MutationResponse, QueryArg, QueryRequest, Sub,
    },
};

use super::proto::{encode_proto, ProtoPackage};

pub trait HttpProtoResponse {
    fn protobuf(&mut self, proto_package: ProtoPackage) -> HttpResponse;
}

impl HttpProtoResponse for HttpResponseBuilder {
    fn protobuf(&mut self, proto_package: ProtoPackage) -> HttpResponse {
        self.insert_header(("Content-Type", "application/protobuf"))
            .insert_header(("0", proto_package.signature))
            .body(proto_package.data)
    }
}

#[cfg_attr(test, automock)]
pub trait RequestHandler<T = ProtoPackage> {
    fn handle_fetch(
        &self,
        request_query: &QueryRequest,
    ) -> impl Future<Output = Result<T, Error>> + Send;
    fn handle_mutate(
        &self,
        request_query: &QueryRequest,
    ) -> impl Future<Output = Result<T, Error>> + Send;
    fn handle_migrate(
        &self,
        migration: &MigrationRequest,
    ) -> impl Future<Output = Result<T, Error>> + Send;
}

pub struct ProtoPackageResultHandler<'a> {
    pub user_access: u8,
    pub username_password_hash: &'a str,
    pub db: &'a SqlitePool,
}

impl<'a> ProtoPackageResultHandler<'a> {
    pub fn new(user_access: u8, username_password_hash: &'a str, db: &'a SqlitePool) -> Self {
        ProtoPackageResultHandler {
            user_access,
            username_password_hash,
            db,
        }
    }
}

impl<'a> RequestHandler<ProtoPackage> for ProtoPackageResultHandler<'a> {
    fn handle_fetch(
        &self,
        request_query: &QueryRequest,
    ) -> impl Future<Output = Result<ProtoPackage, Error>> + Send {
        async move {
            if self.user_access >= 1 {
                match fetch_all_as_json(
                    AppliedQuery::new(&request_query.query).with_args(&request_query.parts),
                    &self.db,
                )
                .await
                {
                    Ok(res) => encode_proto(
                        FetchResponse::as_dat(serde_json::to_vec(&res).unwrap()),
                        Sub::Data,
                        self.username_password_hash,
                    ),
                    Err(e) => Err(DatabaseError::with_message(
                        e.as_database_error().unwrap().message(),
                    )),
                }
            } else {
                Err(UserNotAllowedError::default())
            }
        }
    }

    fn handle_mutate(
        &self,
        request_query: &QueryRequest,
    ) -> impl Future<Output = Result<ProtoPackage, Error>> + Send {
        async move {
            if self.user_access >= 2 {
                let mut transaction = self.db.begin().await.unwrap();
                match execute_query(
                    AppliedQuery::new(&request_query.query).with_args(&request_query.parts),
                    &mut *transaction,
                )
                .await
                {
                    Ok(res) => {
                        let _ = &mut transaction.commit().await;
                        encode_proto(
                            MutationResponse::as_dat(
                                res.rows_affected(),
                                res.last_insert_rowid() as u64,
                            ),
                            Sub::Data,
                            self.username_password_hash,
                        )
                    }
                    Err(e) => {
                        let _ = &mut transaction.rollback().await;
                        Err(DatabaseError::with_message(
                            e.as_database_error().unwrap().message(),
                        ))
                    }
                }
            } else {
                Err(UserNotAllowedError::default())
            }
        }
    }

    fn handle_migrate(
        &self,
        migration: &MigrationRequest,
    ) -> impl Future<Output = Result<ProtoPackage, Error>> + Send {
        async move {
            if self.user_access >= 2 {
                let mut transaction = self.db.begin().await.unwrap();

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
                            return Err(DatabaseError::with_message(
                                e.as_database_error().unwrap().message(),
                            ));
                        }
                    }
                    Err(e) => {
                        let _ = &mut transaction.rollback().await;
                        return Err(DatabaseError::with_message(
                            e.as_database_error().unwrap().message(),
                        ));
                    }
                };

                match execute_query(AppliedQuery::new(&migration.query), &mut *transaction).await {
                    Ok(_) => {
                        let _ = transaction.commit().await;
                        encode_proto(
                            MigrationResponse::as_dat(true),
                            Sub::Data,
                            self.username_password_hash,
                        )
                    }
                    Err(e) => {
                        let _ = transaction.rollback().await;
                        eprintln!("{e}");
                        encode_proto(
                            MigrationResponse::as_dat(false),
                            Sub::Data,
                            self.username_password_hash,
                        )
                    }
                }
            } else {
                Err(UserNotAllowedError::default())
            }
        }
    }
}

pub async fn get_proto_package_result<'a, T>(
    claims: Claims,
    handler: &'a T,
) -> Result<ProtoPackage, Error>
where
    T: RequestHandler,
{
    match &claims.dat {
        Some(Dat::MigrationRequest(dat)) => match claims.sub() {
            Sub::Migrate => handler.handle_migrate(dat).await,
            _ => Err(UndefinedError::default()),
        },
        Some(Dat::QueryRequest(dat)) => match claims.sub() {
            Sub::Mutate => handler.handle_mutate(dat).await,
            Sub::Fetch => handler.handle_fetch(dat).await,
            _ => Err(UndefinedError::default()),
        },
        _ => Err(UndefinedError::default()),
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

pub fn extract_headers<'a>(req: &'a HttpRequest) -> Result<(&'a str, &'a str), Error> {
    let header_username_hash = get_header_value(req.headers().get("0"))?;
    let header_proto_signature = get_header_value(req.headers().get("1"))?;

    Ok((header_username_hash, header_proto_signature))
}
