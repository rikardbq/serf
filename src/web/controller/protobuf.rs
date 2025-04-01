use std::fmt::LowerHex;

// testing purpose for proto
use actix_web::{
    get, http::StatusCode, post, web, HttpRequest, HttpResponse, HttpResponseBuilder, Responder,
    Result,
};
use hmac::{Hmac, Mac};
use prost::Message;
use serde_json::{json, Value};
use serf_proto::{claims, Claims, FetchResponse, MutationResponse, Request};
// use request::MutationResponse;
use sha2::{Digest, Sha256};

use crate::{
    core::{
        constants::queries,
        db::{execute_query, AppliedQuery},
        error::{SerfError, UndefinedError, UserNotAllowedError},
        serf_proto,
        state::AppState,
        util::get_or_insert_db_connection,
    },
    web::{
        proto,
        request::{RequestBody, ResponseResult},
        util::get_header_value,
    },
};

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(handle_testing_proto);
    // cfg.service(handle_test_db_post);
}

// assuming real controller method
// a header called 0 and 1 contain username hash and data signature hash respectively is supplied
// 1 is used to fetch the app state username and password hash in the lookup table
// 0 is used to verify the signature of the data using the username and password hash as a secret when creating the sha256 hash of the input data
// after this is done and either error has been handled or proceeded to decode the protobuf byte array
// the database data is then encoded as protobuf and signed using the server-side known username and password hash as a secret
// with response headers Content-Type: application/protobuf and 0: data_signature value
// consumer-side verifies the returned data with the consumer-side known username and password hash and compares it to the signature from the server
// if it's a match then the consumer knows the data hasn't been tampered with
#[post("/test/testing_proto")]
async fn handle_testing_proto(
    req: HttpRequest,
    data: web::Data<AppState>,
    req_body: web::Bytes,
) -> impl Responder {
    println!("hello world");
    // println!("Raw received bytes: {:?}", req_body);
    let signature_header = match get_header_value(req.headers().get("1")) {
        Ok(header_val) => header_val,
        Err(e) => return HttpResponse::BadRequest().body("asd"),
    };

    let body = proto::decode_proto(req_body.to_vec(), "secret", signature_header.to_string());

    // let mut resp = MutationResponse::default();
    // resp.last_insert_row_id = 123;
    // resp.rows_affected = 1;

    // let mut asdg = Claims::default();
    // asdg.dat = Some(serf_proto::claims::Dat::QueryRequest(
    //     serf_proto::QueryRequest::default(),
    // ));

    let mut response_proto = Request::default();
    let mut claims = Claims::default();
    let mut fetch_resp = FetchResponse::default();
    let mut json_array = Value::Array(vec![]);
    let rows = vec![
        json!({"im_data": "daataaaaaa","im_data_too": "asdasdasdasdf", "im_data_also": "123"}),
        json!({"im_data": "daata22","im_data_too": "chungus", "im_data_also": "432"}),
        json!({"im_data": "yooooo","im_data_too": "big", "im_data_also": "77"}),
    ];

    if let Value::Array(ref mut arr) = json_array {
        arr.extend(rows);
    }

    //     // must create some custom set of functions to make the building of the responses easier
    //     // see sqlite_server_connector_java
    //     // ProtoPackage struct, ProtoPackageVerifier struct?, ProtoPackageUtil file/mod?, ProtoManager file
    //     // signing and encoding/decoding should be easy and should require minimal duplication of work

    //     // for the response builder impl a protobuf function to take a Message trait child type that will
    //     // set the body and base headers needed, maybe utilizing a set of functions that manage the whole signing and/or
    //     // serialization/deserialization of the protobuf message.

    fetch_resp.data = serde_json::to_vec(&json_array).unwrap();
    // let ee = FetchResponse::as_dat(serde_json::to_vec(&json_array).unwrap());
    // let asdf = proto::encode_proto(ee, serf_proto::Sub::Data, "my_secret_key".to_string());

    claims.iss = serf_proto::Iss::Server.into();
    claims.sub = serf_proto::Sub::Data.into();
    claims.iat = chrono::Utc::now().timestamp() as u64;
    claims.exp = (chrono::Utc::now() + chrono::Duration::seconds(30)).timestamp() as u64;
    claims.dat = Some(serf_proto::claims::Dat::FetchResponse(fetch_resp));
    response_proto.claims = Some(claims);

    let mut buf = Vec::new();
    buf.reserve(response_proto.encoded_len());
    response_proto.encode(&mut buf).unwrap();

    let mut mac =
        Hmac::<Sha256>::new_from_slice("my_secret_key".as_bytes()).expect("Something went wrong!");
    mac.update(&buf);
    let result = mac.finalize();
    let result_bytes = result.into_bytes();

    let data_signature = base16ct::lower::encode_string(&result_bytes);

    println!("{}", data_signature);

    HttpResponse::Ok()
        .insert_header(("Content-Type", "application/protobuf"))
        .insert_header(("0", data_signature))
        .body(buf)
}

// #[post("/test/{database}")]
// async fn handle_test_db_post(
//     req: HttpRequest,
//     data: web::Data<AppState>,
//     path: web::Path<String>,
//     req_body: web::Bytes,
// ) -> impl Responder {

//     let response_builder = HttpResponseBuilder::new(StatusCode::OK)
//         .insert_header(("Content-Type", "application/protobuf"));
//     let body = match Request::decode(&mut &req_body[..]) {
//         Ok(body_val) => body_val,
//         Err(e) => return response_builder.status(StatusCode::BAD_REQUEST).body("body"),
//     };

//     let header_username_hash = match get_header_value(req.headers().get("u_")) {
//         Ok(header_val) => header_val,
//         Err(e) => return HttpResponse::BadRequest().json(ResponseResult::new().error(e)),
//     };

//     let db_name = path.into_inner();
//     let users_guard = data.users_guard();
//     let user = match data.get_user(header_username_hash, &users_guard) {
//         Ok(u) => u,
//         Err(e) => return HttpResponse::Unauthorized().json(ResponseResult::new().error(e)),
//     };

//     let payload_token = &req_body.payload;
//     let decoded_token = match decode_token(&payload_token, &user.username_password_hash) {
//         Ok(dec) => dec,
//         Err(_) => {
//             return HttpResponse::InternalServerError()
//                 .json(ResponseResult::new().error(UndefinedError::default()))
//         }
//     };

//     let db_connections_guard = data.db_connections_guard();
//     let db = match get_or_insert_db_connection(&db_connections_guard, &data, &db_name).await {
//         Ok(conn) => conn,
//         Err(e) => return HttpResponse::NotFound().json(ResponseResult::new().error(e)),
//     };

//     let query_result_claims =
//         match get_query_result_claims(decoded_token.claims, user.get_access_right(&db_name), &db)
//             .await
//         {
//             Ok(res) => res,
//             Err(e) => match e.source {
//                 ErrorKind::UserNotAllowed => {
//                     return HttpResponse::Forbidden().json(ResponseResult::new().error(e))
//                 }
//                 _ => {
//                     return HttpResponse::InternalServerError().json(ResponseResult::new().error(e))
//                 }
//             },
//         };

//     let token = match generate_token(query_result_claims, &user.username_password_hash) {
//         Ok(t) => t,
//         Err(_) => {
//             return HttpResponse::InternalServerError()
//                 .json(ResponseResult::new().error(UndefinedError::default()))
//         }
//     };

//     HttpResponse::Ok().json(ResponseResult::new().payload(token))
// }
