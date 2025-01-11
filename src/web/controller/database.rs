use actix_web::{post, web, HttpRequest, HttpResponse, Responder};

use crate::{
    core::{
        constants::queries,
        db::{execute_query, AppliedQuery, QueryArg},
        error::{ErrorKind, SerfError, UndefinedError, UserNotAllowedError},
        state::AppState,
        util::get_or_insert_db_connection,
    },
    web::{
        jwt::{decode_token, generate_claims, generate_token, RequestMigration, Sub},
        request::{RequestBody, ResponseResult},
        util::{get_header_value, get_query_result_claims},
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
    let header_username_hash = match get_header_value(req.headers().get("u_")) {
        Ok(header_val) => header_val,
        Err(e) => return HttpResponse::BadRequest().json(ResponseResult::new().error(e)),
    };

    let db_name = path.into_inner();
    let users_guard = data.users_guard();
    let user = match data.get_user(header_username_hash, &users_guard) {
        Ok(u) => u,
        Err(e) => return HttpResponse::Unauthorized().json(ResponseResult::new().error(e)),
    };

    let payload_token = &req_body.payload;
    let decoded_token = match decode_token(&payload_token, &user.username_password_hash) {
        Ok(dec) => dec,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(ResponseResult::new().error(UndefinedError::default()))
        }
    };

    let db_connections_guard = data.db_connections_guard();
    let db = match get_or_insert_db_connection(&db_connections_guard, &data, &db_name).await {
        Ok(conn) => conn,
        Err(e) => return HttpResponse::NotFound().json(ResponseResult::new().error(e)),
    };

    let query_result_claims =
        match get_query_result_claims(decoded_token.claims, user.get_access_right(&db_name), &db)
            .await
        {
            Ok(res) => res,
            Err(e) => match e.source {
                ErrorKind::UserNotAllowed => {
                    return HttpResponse::Forbidden().json(ResponseResult::new().error(e))
                }
                ErrorKind::SubjectInvalid => {
                    return HttpResponse::NotAcceptable().json(ResponseResult::new().error(e))
                }
                _ => {
                    return HttpResponse::InternalServerError().json(ResponseResult::new().error(e))
                }
            },
        };

    let token = match generate_token(query_result_claims, &user.username_password_hash) {
        Ok(t) => t,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(ResponseResult::new().error(UndefinedError::default()))
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
    let header_username_hash = match get_header_value(req.headers().get("u_")) {
        Ok(header_val) => header_val,
        Err(e) => {
            return HttpResponse::BadRequest().json(ResponseResult::new().error(e));
        }
    };

    let db_name = path.into_inner();
    let users_guard = data.users_guard();
    let user = match data.get_user(header_username_hash, &users_guard) {
        Ok(u) => u,
        Err(e) => return HttpResponse::Unauthorized().json(ResponseResult::new().error(e)),
    };

    if user.get_access_right(&db_name) < 2 {
        return HttpResponse::Forbidden()
            .json(ResponseResult::new().error(UserNotAllowedError::default()));
    }

    let payload_token = &req_body.payload;
    let decoded_token = match decode_token(&payload_token, &user.username_password_hash) {
        Ok(dec) => dec,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(ResponseResult::new().error(UndefinedError::default()))
        }
    };

    let db_connections_guard = data.db_connections_guard();
    let db = match get_or_insert_db_connection(&db_connections_guard, &data, &db_name).await {
        Ok(conn) => conn,
        Err(e) => return HttpResponse::NotFound().json(ResponseResult::new().error(e)),
    };

    let claims = decoded_token.claims;
    if claims.sub != Sub::MIGRATE {
        return HttpResponse::InternalServerError()
            .json(ResponseResult::new().error(UndefinedError::default()));
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
                Err(e) => {
                    let _ = &mut transaction.rollback().await;
                    panic!("{e}");
                }
            }
        }
        Err(e) => {
            let _ = &mut transaction.rollback().await;
            panic!("{e}");
        }
    };

    // apply migration
    let res = match execute_query(AppliedQuery::new(&migration.query), &mut *transaction).await {
        Ok(_) => {
            let _ = &mut transaction.commit().await;
            generate_claims(true.to_string(), Sub::DATA)
        }
        Err(e) => {
            let _ = &mut transaction.rollback().await;
            println!("{e}");
            generate_claims(false.to_string(), Sub::DATA)
        }
    };

    let token = match generate_token(res, &user.username_password_hash) {
        Ok(t) => t,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(ResponseResult::new().error(UndefinedError::default()));
        }
    };

    HttpResponse::Ok().json(ResponseResult::new().payload(token))
}
