// use chrono;
// use jsonwebtoken::crypto::verify;
// use jsonwebtoken::{
//     decode, encode, Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation,
// };
// use serde::{Deserialize, Serialize};
// use serde_json::Value as JsonValue;

// use crate::core::db::QueryArg;

// #[derive(Deserialize, Serialize, Debug)]
// pub struct QueryRequest {
//     pub query: String,
//     pub parts: Vec<QueryArg>,
// }

// #[derive(Deserialize, Serialize, Debug)]
// pub struct MigrationRequest {
//     pub name: String,
//     pub query: String,
// }

// #[derive(Deserialize, Serialize, Debug)]
// pub struct FetchResponse {
//     pub data: Vec<JsonValue>,
// }

// #[derive(Deserialize, Serialize, Debug)]
// pub struct MutationResponse {
//     pub rows_affected: u64,
//     pub last_insert_rowid: i64,
// }

// #[derive(Deserialize, Serialize, Debug)]
// pub struct MigrationResponse {
//     pub state: bool,
// }

// #[derive(Deserialize, Serialize, Debug)]
// #[serde(untagged)]
// pub enum DatKind {
//     FetchResponse(FetchResponse),
//     MutationResponse(MutationResponse),
//     QueryRequest(QueryRequest),
//     MigrationRequest(MigrationRequest),
//     MigrationResponse(MigrationResponse),
// }

// impl QueryRequest {
//     pub fn as_dat_kind(query: String, parts: Vec<QueryArg>) -> DatKind {
//         DatKind::QueryRequest(QueryRequest { query, parts })
//     }
// }

// impl MigrationRequest {
//     pub fn as_dat_kind(name: String, query: String) -> DatKind {
//         DatKind::MigrationRequest(MigrationRequest { name, query })
//     }
// }

// impl FetchResponse {
//     pub fn as_dat_kind(data: Vec<JsonValue>) -> DatKind {
//         DatKind::FetchResponse(FetchResponse { data })
//     }
// }

// impl MutationResponse {
//     pub fn as_dat_kind(rows_affected: u64, last_insert_rowid: i64) -> DatKind {
//         DatKind::MutationResponse(MutationResponse {
//             rows_affected,
//             last_insert_rowid,
//         })
//     }
// }

// impl MigrationResponse {
//     pub fn as_dat_kind(state: bool) -> DatKind {
//         DatKind::MigrationResponse(MigrationResponse { state })
//     }
// }

// #[derive(PartialEq, Serialize, Deserialize, Debug)]
// pub enum Iss {
//     CLIENT,
//     SERVER,
// }

// #[derive(PartialEq, Serialize, Deserialize, Debug)]
// pub enum Sub {
//     DATA,
//     FETCH,
//     MIGRATE,
//     MUTATE,
// }

// #[derive(Serialize, Deserialize, Debug)]
// pub struct Claims {
//     pub iss: Iss,
//     pub sub: Sub,
//     // pub aud: String,
//     pub dat: DatKind,
//     pub iat: usize,
//     pub exp: usize,
// }

// pub fn generate_claims(content: DatKind, subject: Sub) -> Claims {
//     let claims = Claims {
//         iss: Iss::SERVER,
//         sub: subject,
//         // aud: String::from("c_"),
//         dat: content,
//         iat: chrono::Utc::now().timestamp() as usize,
//         exp: (chrono::Utc::now() + chrono::Duration::seconds(30)).timestamp() as usize,
//     };

//     claims
// }

// pub fn generate_token(claims: Claims, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
//     encode(
//         &Header::new(Algorithm::HS256),
//         &claims,
//         &EncodingKey::from_secret(secret.as_bytes()),
//     )
// }

// pub fn verify_token(token: &str, secret: &str) -> Result<bool, jsonwebtoken::errors::Error> {
//     let token_parts: Vec<&str> = token.split(".").collect();

//     let head = token_parts[0];
//     let claim = token_parts[1];
//     let sig = token_parts[2];

//     verify(
//         sig,
//         format!("{}.{}", head, claim).as_bytes(),
//         &DecodingKey::from_secret(secret.as_bytes()),
//         Algorithm::HS256,
//     )
// }

// pub fn decode_token(
//     token: &str,
//     secret: &str,
// ) -> Result<TokenData<Claims>, jsonwebtoken::errors::Error> {
//     let mut validation = Validation::new(Algorithm::HS256);
//     let _ = &mut validation.set_issuer(&["CLIENT"]);

//     // validation.set_audience(&["c_"]); // may need to use this at some point

//     decode::<Claims>(
//         token,
//         &DecodingKey::from_secret(secret.as_bytes()),
//         &mut validation,
//     )
// }
