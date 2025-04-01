// use std::sync::Arc;

// use actix_web::{post, web, HttpRequest, HttpResponse, Responder};

// use crate::{
//     core::state::{AppState, User},
//     web::{
//         jwt::{
//             decode_token, generate_claims, generate_token, verify_token, DatKind, QueryRequest, Sub,
//         },
//         request::{RequestBody, ResponseResult},
//         util::get_header_value,
//     },
// };

// pub fn init(cfg: &mut web::ServiceConfig) {
//     cfg.service(handle_generate_token);
//     cfg.service(handle_verify_token);
//     cfg.service(handle_decode_token);
// }

// #[post("/jwt/generate_token")]
// async fn handle_generate_token(
//     req: HttpRequest,
//     data: web::Data<AppState>,
//     req_body: String,
// ) -> impl Responder {
//     let header_username_hash = match get_header_value(req.headers().get("u_")) {
//         Ok(header_val) => header_val,
//         Err(err) => {
//             return HttpResponse::BadRequest().json(ResponseResult::new().error(err));
//         }
//     };

//     // get the users data
//     let users_clone: Arc<papaya::HashMap<String, User>> = Arc::clone(&data.users);
//     let users = users_clone.pin();
//     let user_entry_for_id = match users.get(header_username_hash) {
//         Some(u) => u,
//         None => {
//             return HttpResponse::Unauthorized()
//                 .json(ResponseResult::new().error("ERROR=UnknownUser"))
//         }
//     };

//     let query_request: QueryRequest = serde_json::from_str(&req_body).unwrap();
//     println!("{query_request:?}");
//     let claims = generate_claims(DatKind::QueryRequest(query_request), Sub::FETCH);
//     let token = generate_token(claims, &user_entry_for_id.username_password_hash).unwrap();

//     return HttpResponse::Ok().body(token);
// }

// #[post("/jwt/verify_token")]
// pub async fn handle_verify_token(
//     req: HttpRequest,
//     data: web::Data<AppState>,
//     req_body: web::Json<RequestBody>,
// ) -> impl Responder {
//     let header_username_hash = match get_header_value(req.headers().get("u_")) {
//         Ok(header_val) => header_val,
//         Err(err) => {
//             return HttpResponse::BadRequest().json(ResponseResult::new().error(err));
//         }
//     };

//     // get the users data
//     let users_clone: Arc<papaya::HashMap<String, User>> = Arc::clone(&data.users);
//     let users = users_clone.pin();
//     let user_entry_for_id = match users.get(header_username_hash) {
//         Some(u) => u,
//         None => {
//             return HttpResponse::Unauthorized()
//                 .json(ResponseResult::new().error("ERROR=UnknownUser"))
//         }
//     };

//     let token = &req_body.payload;
//     let is_token_valid = verify_token(token, &user_entry_for_id.username_password_hash).unwrap();
//     println!("valid={}", is_token_valid);

//     HttpResponse::Ok().body(is_token_valid.to_string())
// }

// #[post("/jwt/decode_token")]
// pub async fn handle_decode_token(
//     req: HttpRequest,
//     data: web::Data<AppState>,
//     req_body: web::Json<RequestBody>,
// ) -> impl Responder {
//     let header_username_hash = match get_header_value(req.headers().get("u_")) {
//         Ok(header_val) => header_val,
//         Err(err) => {
//             return HttpResponse::BadRequest().json(ResponseResult::new().error(err));
//         }
//     };

//     // get the users data
//     let users_clone: Arc<papaya::HashMap<String, User>> = Arc::clone(&data.users);
//     let users = users_clone.pin();
//     let user_entry_for_id = match users.get(header_username_hash) {
//         Some(u) => u,
//         None => {
//             return HttpResponse::Unauthorized()
//                 .json(ResponseResult::new().error("ERROR=UnknownUser"))
//         }
//     };

//     let token = &req_body.payload;
//     let decoded_token = decode_token(token, &user_entry_for_id.username_password_hash).unwrap();
//     println!("token={:?}", decoded_token);

//     HttpResponse::Ok().json(decoded_token.claims)
// }
