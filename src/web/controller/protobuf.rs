use std::fmt::LowerHex;

// testing purpose for proto
use actix_web::{
    get, post, web, HttpRequest, HttpResponse, HttpResponseBuilder, Responder, Result,
};
use hmac::{Hmac, Mac};
use prost::Message;
use serf_proto::{claims, Claims, MutationResponse, Request};
// use request::MutationResponse;
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

pub mod serf_proto {
    include!(concat!(env!("OUT_DIR"), "/serf_proto.rs"));
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(handle_testing_proto);
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
async fn handle_testing_proto(req: HttpRequest, data: web::Data<AppState>, req_body: web::Bytes) -> impl Responder {
    println!("hello world");
    println!("Raw received bytes: {:?}", req_body);
    match Request::decode(&mut &req_body[..]) {
        Ok(body) => {
            println!("Decoded Protobuf: {:?}", body);
            if let Some(c) = body.claims {
                if let Some(d) = c.dat {
                    if let claims::Dat::QueryRequest(dat) = d {
                        println!("{:?}", dat.parts);
                        println!("{:?}", dat.query)
                    }
                }
            }
        },
        Err(e) => {
            println!("Failed to decode Protobuf: {:?}", e);
        }
    }
    
    
    let mut resp = MutationResponse::default();
    resp.last_insert_row_id = 123;
    resp.rows_affected = 1;

    let mut asdg = Claims::default();
    asdg.dat = Some(serf_proto::claims::Dat::QueryRequest(
        serf_proto::QueryRequest::default(),
    ));

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
