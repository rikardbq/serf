use std::{
    collections::HashMap,
    sync::{Arc, Mutex, PoisonError},
};

use crate::{
    core::state::{AppState, Usr},
    web::{
        jwt::{decode_token, generate_token, verify_token, Claims},
        request::{RequestBody, ResponseResult},
    },
};
use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use sqlx::SqlitePool;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(handle_generate_token);
    cfg.service(handle_verify_token);
    cfg.service(handle_decode_token);
}

#[post("/generate_token}")]
async fn handle_generate_token(
    req: HttpRequest,
    data: web::Data<AppState<SqlitePool>>,
    req_body: String,
) -> impl Responder {
    let header_u_ = match req.headers().get("u_") {
        Some(hdr) => hdr.to_str().unwrap(),
        _ => {
            return HttpResponse::BadRequest()
                .json(ResponseResult::new().error("ERROR=MissingHeader/s[ u_ ]"));
        }
    };

    // get the usr data
    let usr_clone: Arc<Mutex<HashMap<String, Usr>>> = Arc::clone(&data.usr);
    let usr = usr_clone.lock().unwrap_or_else(PoisonError::into_inner);

    let user_entry_for_id = &usr[header_u_];
    
    let claims = Claims {
        iss: String::from("s_"),
        sub: String::from("d_"),
        dat: req_body,
        iat: chrono::Utc::now().timestamp() as usize,
        exp: (chrono::Utc::now() + chrono::Duration::seconds(30)).timestamp() as usize,
    };

    let token = generate_token(claims, &user_entry_for_id.up_hash).unwrap();

    return HttpResponse::Ok().body(token);
}

#[post("/verify_token")]
pub async fn handle_verify_token(
    req: HttpRequest,
    data: web::Data<AppState<SqlitePool>>,
    req_body: web::Json<RequestBody>,
) -> impl Responder {
    let header_u_ = match req.headers().get("u_") {
        Some(hdr) => hdr.to_str().unwrap(),
        _ => {
            return HttpResponse::BadRequest()
                .json(ResponseResult::new().error("ERROR=MissingHeader/s[ u_ ]"));
        }
    };

    // get the usr data
    let usr_clone: Arc<Mutex<HashMap<String, Usr>>> = Arc::clone(&data.usr);
    let usr = usr_clone.lock().unwrap_or_else(PoisonError::into_inner);

    let user_entry_for_id = &usr[header_u_];

    let token = &req_body.payload;
    let is_token_valid = verify_token(token, &user_entry_for_id.up_hash).unwrap();
    println!("valid={}", is_token_valid);

    HttpResponse::Ok().body(is_token_valid.to_string())
}

#[post("/decode_token")]
pub async fn handle_decode_token(
    req: HttpRequest,
    data: web::Data<AppState<SqlitePool>>,
    req_body: web::Json<RequestBody>,
) -> impl Responder {
    let header_u_ = match req.headers().get("u_") {
        Some(hdr) => hdr.to_str().unwrap(),
        _ => {
            return HttpResponse::BadRequest()
                .json(ResponseResult::new().error("ERROR=MissingHeader/s[ u_ ]"));
        }
    };

    // get the usr data
    let usr_clone: Arc<Mutex<HashMap<String, Usr>>> = Arc::clone(&data.usr);
    let usr = usr_clone.lock().unwrap_or_else(PoisonError::into_inner);

    let user_entry_for_id = &usr[header_u_];

    let token = &req_body.payload;
    let decoded_token = decode_token(token, &user_entry_for_id.up_hash).unwrap();
    println!("token={:?}", decoded_token);

    HttpResponse::Ok().json(decoded_token.claims)
}
