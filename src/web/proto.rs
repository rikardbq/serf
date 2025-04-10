use chrono;
use hmac::{Hmac, Mac};
use prost::Message;
use sha2::Sha256;

use crate::core::{
    error::{ProtoPackageError, SerfError},
    serf_proto::{
        claims::Dat, query_arg, Claims, Error, FetchResponse, Iss, MigrationRequest,
        MigrationResponse, MutationResponse, QueryArg, QueryRequest, Request, Sub,
    },
};

impl QueryArg {
    pub fn new(value: query_arg::Value) -> Self {
        QueryArg { value: Some(value) }
    }
}

impl QueryRequest {
    pub fn as_dat(query: String, parts: Vec<QueryArg>) -> Dat {
        Dat::QueryRequest(QueryRequest { query, parts })
    }
}

impl MigrationRequest {
    pub fn as_dat(name: String, query: String) -> Dat {
        Dat::MigrationRequest(MigrationRequest { name, query })
    }
}

impl FetchResponse {
    pub fn as_dat(data: Vec<u8>) -> Dat {
        Dat::FetchResponse(FetchResponse { data })
    }
}

impl MutationResponse {
    pub fn as_dat(rows_affected: u64, last_insert_row_id: u64) -> Dat {
        Dat::MutationResponse(MutationResponse {
            rows_affected,
            last_insert_row_id,
        })
    }
}

impl MigrationResponse {
    pub fn as_dat(state: bool) -> Dat {
        Dat::MigrationResponse(MigrationResponse { state })
    }
}

#[derive(Eq, PartialEq, Debug)]
pub struct ProtoPackage {
    pub data: Vec<u8>,
    pub signature: String,
}

impl ProtoPackage {
    fn new(data: Vec<u8>, signature: String) -> Self {
        ProtoPackage { data, signature }
    }

    pub fn builder() -> ProtoPackageBuilder {
        ProtoPackageBuilder::new()
    }
}

pub struct ProtoPackageBuilder {
    data: Option<Dat>,
    subject: Option<Sub>,
    iat: Option<u64>,
    exp: Option<u64>,
    error: Option<Error>,
}

impl ProtoPackageBuilder {
    fn new() -> Self {
        ProtoPackageBuilder {
            data: None,
            subject: None,
            iat: None,
            exp: None,
            error: None,
        }
    }

    pub fn with_data(self, data: Dat) -> Self {
        ProtoPackageBuilder {
            data: Some(data),
            ..self
        }
    }

    pub fn with_subject(self, subject: Sub) -> Self {
        ProtoPackageBuilder {
            subject: Some(subject),
            ..self
        }
    }

    pub fn with_error(self, error: Error) -> Self {
        ProtoPackageBuilder {
            error: Some(error),
            ..self
        }
    }

    pub fn with_iat(self, iat: u64) -> Self {
        ProtoPackageBuilder {
            iat: Some(iat),
            ..self
        }
    }

    pub fn with_exp(self, exp: u64) -> Self {
        ProtoPackageBuilder {
            exp: Some(exp),
            ..self
        }
    }

    // ToDo: Fix this to maybe be a little cleaner. Quick solution for now
    pub fn sign(self, secret: &str) -> Result<ProtoPackage, Error> {
        let request: Request;

        if self.error.is_some() {
            request = Request {
                claims: None,
                error: self.error,
            }
        } else if self.subject.is_some() {
            if self.data.is_some() {
                let iat = match self.iat {
                    Some(t) => t,
                    _ => chrono::Utc::now().timestamp() as u64,
                };
                
                let exp = match self.exp {
                    Some(t) => t,
                    _ => iat + 30,
                };

                let claims = generate_claims(self.data.unwrap(), self.subject.unwrap(), iat, exp);
                request = Request {
                    claims: Some(claims),
                    error: None,
                };
            } else {
                return Err(ProtoPackageError::signing_error("missing data"));
            }
        } else {
            return Err(ProtoPackageError::signing_error("missing subject"));
        }

        let mut buf = Vec::new();
        buf.reserve(request.encoded_len());

        if let Err(e) = request.encode(&mut buf) {
            eprintln!("{e}");
            return Err(ProtoPackageError::with_message(
                "PROTOBUF::ENCODE: request proto could not be encoded",
            ));
        }

        let signature = generate_signature(&buf, secret.as_bytes());

        Ok(ProtoPackage::new(buf, signature))
    }
}

pub struct ProtoPackageVerifier<'a> {
    signature: Option<&'a str>,
    secret: Option<&'a str>,
    // subject: Option<Sub>,
    issuer: Option<Iss>,
    now: Option<u64>,
}

impl<'a> ProtoPackageVerifier<'a> {
    fn new(
        signature: Option<&'a str>,
        secret: Option<&'a str>,
        // subject: Option<Sub>,
        issuer: Option<Iss>,
        now: Option<u64>,
    ) -> Self {
        ProtoPackageVerifier {
            signature,
            secret,
            // subject,
            issuer,
            now,
        }
    }

    pub fn verify(self, data: &[u8]) -> Result<Request, Error> {
        if self.signature.is_none() {
            return Err(ProtoPackageError::verification_error("missing signature"));
        }

        if self.secret.is_none() {
            return Err(ProtoPackageError::verification_error("missing secret"));
        }

        if !verify_signature(
            data,
            self.signature.unwrap(),
            self.secret.unwrap().as_bytes(),
        ) {
            return Err(ProtoPackageError::verification_error("invalid signature"));
        }

        let decoded = match Request::decode(&mut &data[..]) {
            Ok(d) => d,
            Err(_) => {
                return Err(ProtoPackageError::with_message(
                    "PROTOBUF::DECODE: request proto could not be decoded",
                ));
            }
        };

        if let Some(claims) = &decoded.claims {
            if !Sub::is_valid(claims.sub) {
                return Err(ProtoPackageError::verification_error(
                    "invalid claims subject",
                ));
            }

            if let Some(issuer) = self.issuer {
                if issuer != claims.iss() {
                    return Err(ProtoPackageError::verification_error(
                        "invalid claims issuer",
                    ));
                }
            }

            if claims.dat.is_none() {
                return Err(ProtoPackageError::verification_error("missing claims data"));
            }

            let now = match self.now {
                Some(n) => n,
                _ => chrono::Utc::now().timestamp() as u64,
            };

            if now > claims.exp {
                return Err(ProtoPackageError::verification_error("claims expired"));
            }
        } else {
            return Err(ProtoPackageError::verification_error("missing claims"));
        }

        Ok(decoded)
    }

    pub fn builder() -> ProtoPackageVerifierBuilder<'a> {
        ProtoPackageVerifierBuilder::new()
    }
}

pub struct ProtoPackageVerifierBuilder<'a> {
    signature: Option<&'a str>,
    secret: Option<&'a str>,
    // subject: Option<Sub>,
    issuer: Option<Iss>,
    now: Option<u64>,
}

impl<'a> ProtoPackageVerifierBuilder<'a> {
    fn new() -> Self {
        ProtoPackageVerifierBuilder {
            signature: None,
            secret: None,
            // subject: None,
            issuer: None,
            now: None,
        }
    }

    pub fn with_signature(self, signature: &'a str) -> Self {
        ProtoPackageVerifierBuilder {
            signature: Some(signature),
            ..self
        }
    }

    pub fn with_secret(self, secret: &'a str) -> Self {
        ProtoPackageVerifierBuilder {
            secret: Some(secret),
            ..self
        }
    }

    // pub fn with_subject(self, subject: Sub) -> Self {
    //     ProtoPackageVerifierBuilder {
    //         subject: Some(subject),
    //         ..self
    //     }
    // }

    pub fn with_issuer(self, issuer: Iss) -> Self {
        ProtoPackageVerifierBuilder {
            issuer: Some(issuer),
            ..self
        }
    }

    #[allow(non_snake_case)]
    #[cfg(test)]
    pub fn TEST_with_now_timestamp(self, now: u64) -> Self {
        ProtoPackageVerifierBuilder {
            now: Some(now),
            ..self
        }
    }

    pub fn build(self) -> ProtoPackageVerifier<'a> {
        ProtoPackageVerifier::new(self.signature, self.secret, self.issuer, self.now)
    }
}

fn verify_signature(data: &[u8], signature: &str, secret: &[u8]) -> bool {
    signature == generate_signature(data, secret)
}

pub fn generate_signature(data: &[u8], secret: &[u8]) -> String {
    let mut mac = match Hmac::<Sha256>::new_from_slice(&secret) {
        Ok(m) => m,
        Err(_) => panic!("ERROR DURING SIGNING"),
    };

    mac.update(&data);
    let result = mac.finalize();
    let result_bytes = result.into_bytes();

    base16ct::lower::encode_string(&result_bytes)
}

fn generate_claims(data: Dat, subject: Sub, iat: u64, exp: u64) -> Claims {
    Claims {
        iss: Iss::Server.into(),
        sub: subject.into(),
        dat: Some(data),
        iat,
        exp, //(chrono::Utc::now() + chrono::Duration::seconds(30)).timestamp() as u64,
    }
}

pub fn encode_proto(data: Dat, subject: Sub, secret: &str) -> Result<ProtoPackage, Error> {
    let proto_package_builder = ProtoPackage::builder();

    proto_package_builder
        .with_data(data)
        .with_subject(subject)
        .sign(secret)
}

pub fn encode_error_proto(error: Error, secret: &str) -> ProtoPackage {
    let proto_package_builder = ProtoPackage::builder();

    proto_package_builder
        .with_error(error)
        .sign(secret)
        .unwrap()
}

pub fn decode_proto(proto_bytes: &[u8], secret: &str, signature: &str) -> Result<Request, Error> {
    let proto_package_verifier = ProtoPackageVerifier::builder()
        .with_signature(signature)
        .with_secret(secret)
        .with_issuer(Iss::Client)
        .build();

    proto_package_verifier.verify(proto_bytes)
}
