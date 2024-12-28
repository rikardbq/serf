use actix_web::{post, web, HttpRequest, HttpResponse, Responder};

use crate::{
    core::{
        constants::{
            errors::{self, ErrorReason},
            queries,
        },
        db::{execute_query, fetch_all_as_json, AppliedQuery, QueryArg},
        state::AppState,
        util::{get_db_connections, get_user_entry_and_access, Error},
    },
    web::{
        jwt::{decode_token, generate_claims, generate_token, RequestMigration, RequestQuery, Sub},
        request::{RequestBody, ResponseResult},
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
    req_body: web::Json<RequestBody>,
) -> impl Responder {
    let header_u_ = match req.headers().get("u_") {
        Some(hdr) => hdr.to_str().unwrap(),
        _ => {
            return HttpResponse::BadRequest().json(ResponseResult::new().error(format!(
                "{} [ {} ]",
                errors::ERROR_MISSING_HEADER,
                "u_"
            )));
        }
    };

    let db_name = path.into_inner();
    let (user_entry, user_access) =
        match get_user_entry_and_access(&data, header_u_, &db_name) {
            Ok(ue) => ue,
            Err(err) => return HttpResponse::Unauthorized().json(ResponseResult::new().error(err)),
        };

    let payload_token = &req_body.payload;
    let decoded_token = match decode_token(&payload_token, &user_entry.username_password_hash) {
        Ok(dec) => dec,
        Err(err) => {
            return HttpResponse::NotAcceptable()
                .json(ResponseResult::new().error(&format!("ERROR={:?}", err.kind())))
        }
    };

    let db = match get_db_connections(&data, &db_name).await {
        Ok(conn) => conn,
        Err(err) => return HttpResponse::NotFound().json(ResponseResult::new().error(err)),
    };

    let query_result_claims =
        match get_query_result_claims(decoded_token.claims, user_access, &db).await {
            Ok(res) => res,
            Err(err) => {
                if let Some(reason) = err.reason {
                    if reason == ErrorReason::UserNotAllowed {
                        return HttpResponse::Forbidden()
                            .json(ResponseResult::new().error(err.message));
                    } else if reason == ErrorReason::InvalidSubject {
                        return HttpResponse::NotAcceptable()
                            .json(ResponseResult::new().error(err.message));
                    }
                }

                return HttpResponse::InternalServerError()
                    .json(ResponseResult::new().error(err.message));
            }
        };

    let token = match generate_token(query_result_claims, &user_entry.up_hash) {
        Ok(t) => t,
        Err(err) => {
            return HttpResponse::InternalServerError()
                .json(ResponseResult::new().error(&format!("ERROR={:?}", err.kind())))
        }
    };

    HttpResponse::Ok().json(ResponseResult::new().payload(token))
}

#[post("/{database}/m")]
async fn handle_db_migration_post(
    req: HttpRequest,
    data: web::Data<AppState>,
    path: web::Path<String>,
    req_body: web::Json<RequestBody>,
) -> impl Responder {
    let header_u_ = match req.headers().get("u_") {
        Some(hdr) => hdr.to_str().unwrap(),
        _ => {
            return HttpResponse::BadRequest().json(ResponseResult::new().error(format!(
                "{} [ {} ]",
                errors::ERROR_MISSING_HEADER,
                "u_"
            )));
        }
    };

    let db_name = path.into_inner();
    let (user_entry, user_access) =
        match get_user_entry_and_access(&data, header_u_, &db_name) {
            Ok(ue) => ue,
            Err(err) => return HttpResponse::Unauthorized().json(ResponseResult::new().error(err)),
        };

    if user_access < 2 {
        return HttpResponse::Forbidden()
            .json(ResponseResult::new().error(errors::ERROR_USER_NOT_ALLOWED));
    }

    let payload_token = &req_body.payload;
    let decoded_token = match decode_token(&payload_token, &user_entry.username_password_hash) {
        Ok(dec) => dec,
        Err(err) => {
            return HttpResponse::NotAcceptable()
                .json(ResponseResult::new().error(&format!("ERROR={:?}", err.kind())))
        }
    };

    let db = match get_db_connections(&data, &db_name).await {
        Ok(conn) => conn,
        Err(err) => return HttpResponse::NotFound().json(ResponseResult::new().error(err)),
    };

    let claims = decoded_token.claims;
    if claims.sub != Sub::MIGRATE {
        return HttpResponse::NotAcceptable()
            .json(ResponseResult::new().error(errors::ERROR_INVALID_SUBJECT));
    }

    let migration: RequestMigration = serde_json::from_str(&claims.dat).unwrap();

    // start transaction here because we want to fail everything if we can't insert migration state
    // or if we fail any of these steps in any way
    let mut transaction = db.begin().await.unwrap();

    // create migration table and panic if fail
    match execute_query(
        AppliedQuery::new(queries::CREATE_MIGRATIONS_TABLE),
        &mut *transaction,
    )
    .await
    {
        Ok(_) => {
            match execute_query(
                AppliedQuery::new(queries::INSERT_MIGRATION).with_args(vec![
                    QueryArg::String(&migration.name),
                    QueryArg::String(&migration.query),
                ]),
                &mut *transaction,
            )
            .await
            {
                Ok(_) => {}
                Err(err) => {
                    let _ = &mut transaction.rollback().await;
                    panic!("{err}");
                }
            }
        }
        Err(err) => {
            let _ = &mut transaction.rollback().await;
            panic!("{err}");
        }
    };

    // apply migration
    let res = match execute_query(AppliedQuery::new(&migration.query), &mut *transaction).await {
        Ok(_) => {
            let _ = &mut transaction.commit().await;
            generate_claims(true.to_string(), Sub::DATA)
        }
        Err(err) => {
            let _ = &mut transaction.rollback().await;
            println!("{err}");
            generate_claims(false.to_string(), Sub::DATA)
        }
    };
    let token = generate_token(res, &user_entry.username_password_hash).unwrap();

    HttpResponse::Ok().json(ResponseResult::new().payload(token))
}
