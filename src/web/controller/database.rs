use actix_web::{post, web, HttpRequest, HttpResponse, Responder};

use crate::{
    core::{
        constants::queries,
        db::{execute_query, AppliedQuery},
        serf_proto::{
            claims::Dat, query_arg, ErrorKind, MigrationRequest, MigrationResponse, QueryArg,
            Request, Sub,
        },
        state::AppState,
        util::get_or_insert_db_connection,
    },
    web::{
        proto::{decode_proto, encode_proto},
        util::{build_proto_response, get_header_value, get_query_result_proto_package},
    },
};

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(handle_db_post);
    cfg.service(handle_db_migration_post);
}

#[post("/{database}")]
async fn handle_db_post(
    req: HttpRequest,
    data: web::Data<AppState>,
    path: web::Path<String>,
    req_body: web::Bytes,
) -> impl Responder {
    let header_username_hash = match get_header_value(req.headers().get("0")) {
        Ok(val) => val,
        Err(e) => return HttpResponse::BadRequest().body("missing something"),
    };
    let header_proto_signature = match get_header_value(req.headers().get("1")) {
        Ok(val) => val,
        Err(e) => return HttpResponse::BadRequest().body("missing something"),
    };

    let db_name = path.into_inner();
    let users_guard = data.users_guard();
    let user = match data.get_user(header_username_hash, &users_guard) {
        Ok(u) => u,
        Err(e) => {
            // return HttpResponse::Unauthorized().json(ResponseResult::new().error(e))
            return HttpResponse::Unauthorized().body("unauthorized");
        }
    };

    // add error management later, rn just use as is, should return result and propagate errors from the verification process
    let decoded_proto: Request = decode_proto(
        req_body.to_vec(),
        &user.username_password_hash,
        header_proto_signature.to_string(),
    );

    if decoded_proto.claims.is_none() {
        panic!("something went wrong! no claims");
    }

    let claims = decoded_proto.claims.unwrap();

    let db_connections_guard = data.db_connections_guard();
    let db = match get_or_insert_db_connection(&db_connections_guard, &data, &db_name).await {
        Ok(conn) => conn,
        Err(e) => {
            // return HttpResponse::NotFound().json(ResponseResult::new().error(e))
            return HttpResponse::NotFound().body("db not found");
        }
    };

    let proto_package = match get_query_result_proto_package(
        &claims.dat,
        claims.sub(),
        user.get_access_right(&db_name),
        &user.username_password_hash,
        &db,
    )
    .await
    {
        Ok(res) => res,
        Err(e) => match e.source() {
            ErrorKind::UserNotAllowed => {
                // return HttpResponse::Forbidden().json(ResponseResult::new().error(e))
                return HttpResponse::Forbidden().body("no access");
            }
            _ => {
                // return HttpResponse::InternalServerError().json(ResponseResult::new().error(e))
                return HttpResponse::InternalServerError()
                    .body("something went wrong with useraccess??");
            }
        },
    };

    build_proto_response(&mut HttpResponse::Ok(), proto_package)
}

#[post("/{database}/m")]
async fn handle_db_migration_post(
    req: HttpRequest,
    data: web::Data<AppState>,
    path: web::Path<String>,
    req_body: web::Bytes,
) -> impl Responder {
    let header_username_hash = match get_header_value(req.headers().get("0")) {
        Ok(val) => val,
        Err(e) => return HttpResponse::BadRequest().body("missing something"),
    };
    let header_proto_signature = match get_header_value(req.headers().get("1")) {
        Ok(val) => val,
        Err(e) => return HttpResponse::BadRequest().body("missing something"),
    };

    let db_name = path.into_inner();
    let users_guard = data.users_guard();
    let user = match data.get_user(header_username_hash, &users_guard) {
        Ok(u) => u,
        Err(e) => {
            // return HttpResponse::Unauthorized().json(ResponseResult::new().error(e))
            return HttpResponse::Unauthorized().body("unauthorized");
        }
    };

    if user.get_access_right(&db_name) < 2 {
        return HttpResponse::Forbidden().body("user not allowed");
        // return HttpResponse::Forbidden()
        //     .json(ResponseResult::new().error(UserNotAllowedError::default()));
    }

    let decoded_proto: Request = decode_proto(
        req_body.to_vec(),
        &user.username_password_hash,
        header_proto_signature.to_string(),
    );

    if decoded_proto.claims.is_none() {
        panic!("something went wrong! no claims");
    }

    let claims = decoded_proto.claims.unwrap();

    if claims.sub() != Sub::Migrate {
        return HttpResponse::InternalServerError().body("not a migration");
        // return HttpResponse::InternalServerError()
        //     .json(ResponseResult::new().error(UndefinedError::default()));
    }

    let db_connections_guard = data.db_connections_guard();
    let db = match get_or_insert_db_connection(&db_connections_guard, &data, &db_name).await {
        Ok(conn) => conn,
        Err(e) => {
            return HttpResponse::NotFound().body("db not found");
            // return HttpResponse::NotFound().json(ResponseResult::new().error(e))
        }
    };

    let migration: MigrationRequest = match claims.dat {
        Some(Dat::MigrationRequest(migration_request)) => migration_request,
        _ => {
            return HttpResponse::InternalServerError().body("undefined error");
            // return HttpResponse::InternalServerError()
            //     .json(ResponseResult::new().error(UndefinedError::default()));
        }
    };
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
                    QueryArg::new(query_arg::Value::String(migration.name)),
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

    let proto_package =
        match execute_query(AppliedQuery::new(&migration.query), &mut *transaction).await {
            Ok(_) => {
                let _ = &mut transaction.commit().await;
                encode_proto(
                    MigrationResponse::as_dat(true),
                    Sub::Data,
                    &user.username_password_hash,
                )
                // generate_claims(MigrationResponse::as_dat_kind(true), Sub::DATA)
            }
            Err(e) => {
                let _ = &mut transaction.rollback().await;
                eprintln!("{e}");
                encode_proto(
                    MigrationResponse::as_dat(false),
                    Sub::Data,
                    &user.username_password_hash,
                )
                // generate_claims(MigrationResponse::as_dat_kind(false), Sub::DATA)
            }
        };

    build_proto_response(&mut HttpResponse::Ok(), proto_package)
}
