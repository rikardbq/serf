use actix_web::{post, web, HttpRequest, HttpResponse, Responder};

use crate::{
    core::{
        error::{SerfError, UndefinedError},
        serf_proto::{ErrorKind, Request},
        state::AppState,
        util::get_or_insert_db_connection,
    },
    web::{
        proto::{decode_proto, encode_error_proto},
        util::{extract_headers, get_proto_package_result, HttpProtoResponse},
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
    let (header_username_hash, header_proto_signature) = match extract_headers(&req) {
        Ok((val1, val2)) => (val1, val2),
        Err(e) => return HttpResponse::BadRequest().body(e.message),
    };

    let db_name = path.into_inner();
    let users_guard = data.users_guard();
    let user = match data.get_user(header_username_hash, &users_guard) {
        Ok(val) => val,
        Err(e) => {
            return HttpResponse::Unauthorized().body(e.message);
        }
    };

    let decoded_proto: Request = decode_proto(
        req_body.iter().as_slice(),
        &user.username_password_hash,
        header_proto_signature,
    );

    let claims = match decoded_proto.claims {
        Some(c) => c,
        None => {
            return HttpResponse::InternalServerError().protobuf(encode_error_proto(
                UndefinedError::default(),
                &user.username_password_hash,
            ));
        }
    };

    let db_connections_guard = data.db_connections_guard();
    let db = match get_or_insert_db_connection(&db_connections_guard, &data, &db_name).await {
        Ok(conn) => conn,
        Err(e) => {
            return HttpResponse::NotFound()
                .protobuf(encode_error_proto(e, &user.username_password_hash));
        }
    };

    let proto_package = match get_proto_package_result(
        claims,
        user.get_access_right(&db_name),
        &user.username_password_hash,
        &db,
    )
    .await
    {
        Ok(res) => res,
        Err(e) => match e.source() {
            ErrorKind::UserNotAllowed => {
                return HttpResponse::Forbidden()
                    .protobuf(encode_error_proto(e, &user.username_password_hash))
            }
            _ => {
                return HttpResponse::InternalServerError()
                    .protobuf(encode_error_proto(e, &user.username_password_hash))
            }
        },
    };

    HttpResponse::Ok().protobuf(proto_package)
}

#[post("/{database}/m")]
async fn handle_db_migration_post(
    req: HttpRequest,
    data: web::Data<AppState>,
    path: web::Path<String>,
    req_body: web::Bytes,
) -> impl Responder {
    let (header_username_hash, header_proto_signature) = match extract_headers(&req) {
        Ok((val1, val2)) => (val1, val2),
        Err(e) => return HttpResponse::BadRequest().body(e.message),
    };

    let db_name = path.into_inner();
    let users_guard = data.users_guard();
    let user = match data.get_user(header_username_hash, &users_guard) {
        Ok(val) => val,
        Err(e) => {
            return HttpResponse::Unauthorized().body(e.message);
        }
    };

    let decoded_proto: Request = decode_proto(
        req_body.iter().as_slice(),
        &user.username_password_hash,
        header_proto_signature,
    );

    let claims = match decoded_proto.claims {
        Some(c) => c,
        None => {
            return HttpResponse::InternalServerError().protobuf(encode_error_proto(
                UndefinedError::default(),
                &user.username_password_hash,
            ));
        }
    };

    let db_connections_guard = data.db_connections_guard();
    let db = match get_or_insert_db_connection(&db_connections_guard, &data, &db_name).await {
        Ok(conn) => conn,
        Err(e) => {
            return HttpResponse::NotFound()
                .protobuf(encode_error_proto(e, &user.username_password_hash));
        }
    };

    let proto_package = match get_proto_package_result(
        claims,
        user.get_access_right(&db_name),
        &user.username_password_hash,
        &db,
    )
    .await
    {
        Ok(res) => res,
        Err(e) => match e.source() {
            ErrorKind::UserNotAllowed => {
                return HttpResponse::Forbidden()
                    .protobuf(encode_error_proto(e, &user.username_password_hash));
            }
            _ => {
                return HttpResponse::InternalServerError()
                    .protobuf(encode_error_proto(e, &user.username_password_hash));
            }
        },
    };

    HttpResponse::Ok().protobuf(proto_package)
}
