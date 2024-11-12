use std::sync::Arc;

use actix_web::{post, web, HttpRequest, HttpResponse, Responder};

use crate::{
    core::state::{AppState, Usr},
    web::{
        jwt::{decode_token, generate_claims, generate_token, verify_token, Sub},
        request::{RequestBody, ResponseResult},
    },
};

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(handle_generate_token);
    cfg.service(handle_verify_token);
    cfg.service(handle_decode_token);
}

#[post("/jwt/generate_token")]
async fn handle_generate_token(
    req: HttpRequest,
    data: web::Data<AppState>,
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
    let usr_clone: Arc<papaya::HashMap<String, Usr>> = Arc::clone(&data.usr);
    let usr = usr_clone.pin();
    let user_entry_for_id = match usr.get(header_u_) {
        Some(u) => u,
        None => {
            return HttpResponse::Unauthorized()
                .json(ResponseResult::new().error("ERROR=UnknownUser"))
        }
    };

    let claims = generate_claims(req_body, Sub::F_);
    let token = generate_token(claims, &user_entry_for_id.up_hash).unwrap();

    return HttpResponse::Ok().body(token);
}

#[post("/jwt/verify_token")]
pub async fn handle_verify_token(
    req: HttpRequest,
    data: web::Data<AppState>,
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
    let usr_clone: Arc<papaya::HashMap<String, Usr>> = Arc::clone(&data.usr);
    let usr = usr_clone.pin();
    let user_entry_for_id = match usr.get(header_u_) {
        Some(u) => u,
        None => {
            return HttpResponse::Unauthorized()
                .json(ResponseResult::new().error("ERROR=UnknownUser"))
        }
    };

    let token = &req_body.payload;
    let is_token_valid = verify_token(token, &user_entry_for_id.up_hash).unwrap();
    println!("valid={}", is_token_valid);

    HttpResponse::Ok().body(is_token_valid.to_string())
}

#[post("/jwt/decode_token")]
pub async fn handle_decode_token(
    req: HttpRequest,
    data: web::Data<AppState>,
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
    let usr_clone: Arc<papaya::HashMap<String, Usr>> = Arc::clone(&data.usr);
    let usr = usr_clone.pin();
    let user_entry_for_id = match usr.get(header_u_) {
        Some(u) => u,
        None => {
            return HttpResponse::Unauthorized()
                .json(ResponseResult::new().error("ERROR=UnknownUser"))
        }
    };

    let token = &req_body.payload;
    let decoded_token = decode_token(token, &user_entry_for_id.up_hash).unwrap();
    println!("token={:?}", decoded_token);

    HttpResponse::Ok().json(decoded_token.claims)
}
