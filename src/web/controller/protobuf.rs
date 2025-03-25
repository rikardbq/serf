use std::fmt::LowerHex;

// testing purpose for proto
use actix_web::{
    get, post, web, HttpRequest, HttpResponse, HttpResponseBuilder, Responder, Result,
};
use hmac::{Hmac, Mac};
use prost::Message;
use request::MutationResponse;
use sha2::{Digest, Sha256};

use crate::{
    core::{
        constants::queries,
        db::{execute_query, AppliedQuery, QueryArg},
        error::{ErrorKind, SerfError, UndefinedError, UserNotAllowedError},
        state::AppState,
        util::get_or_insert_db_connection,
    },
    web::{
        jwt::{
            decode_token, generate_claims, generate_token, DatKind, MigrationRequest,
            MigrationResponse, Sub,
        },
        request::{RequestBody, ResponseResult},
        util::{get_header_value, get_query_result_claims},
    },
};

pub mod request {
    include!(concat!(env!("OUT_DIR"), "/serf.request.rs"));
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(handle_testing_proto);
}

#[get("/testing_proto")]
async fn handle_testing_proto(req: HttpRequest, data: web::Data<AppState>) -> impl Responder {
    let mut resp = MutationResponse::default();
    resp.last_insert_row_id = 123;
    resp.rows_affected = 1;

    let mut buf = Vec::new();
    buf.reserve(resp.encoded_len());
    resp.encode(&mut buf).unwrap();

    let mut mac = Hmac::<Sha256>::new_from_slice(b"my_secret_key").expect("Something went wrong!");
    mac.update(&buf);
    let result = mac.finalize();
    let result_bytes = result.into_bytes();
    let data_signature = base16ct::lower::encode_string(&result_bytes);

    HttpResponse::Ok()
        .insert_header(("Content-Type", "application/protobuf"))
        .insert_header(("0", data_signature))
        .body(buf)
}
