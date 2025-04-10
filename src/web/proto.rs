use chrono;
use hmac::{Hmac, Mac};
use prost::Message;
use sha2::Sha256;

use crate::core::{
    error::{SerfError, UndefinedError},
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

pub struct ProtoPackage {
    pub data: Vec<u8>,
    pub signature: String,
}

impl ProtoPackage {
    fn new(data: Vec<u8>, signature: String) -> Self {
        ProtoPackage {
            data,
            signature,
        }
    }

    pub fn builder() -> ProtoPackageBuilder {
        ProtoPackageBuilder::new()
    }
}

pub struct ProtoPackageBuilder {
    data: Option<Dat>,
    subject: Option<Sub>,
    error: Option<Error>,
}

impl ProtoPackageBuilder {
    fn new() -> Self {
        ProtoPackageBuilder {
            data: None,
            subject: None,
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

    // ToDo: Fix this to maybe be a little cleaner. Quick solution for now
    pub fn sign(self, secret: &str) -> Result<ProtoPackage, Error> {
        let request: Request;

        if self.error.is_some() {
            request = Request {
                claims: None,
                error: self.error,
            }
        } else if self.subject.is_some() && self.data.is_some() {
            let claims = generate_claims(self.data.unwrap(), self.subject.unwrap());
            request = Request {
                claims: Some(claims),
                error: None,
            };
        } else {
            return Err(UndefinedError::default());
        }

        let mut buf = Vec::new();
        buf.reserve(request.encoded_len());

        if let Err(e) = request.encode(&mut buf) {
            eprintln!("{e}");
            panic!("Error during request proto encoding");
        }

        let signature = generate_signature(&buf, secret.as_bytes());

        Ok(ProtoPackage::new(buf, signature))
    }
}

struct ProtoPackageVerifier<'a> {
    signature: Option<&'a str>,
    secret: Option<&'a str>,
    // subject: Option<Sub>,
    issuer: Option<Iss>,
}

impl<'a> ProtoPackageVerifier<'a> {
    fn new(
        signature: Option<&'a str>,
        secret: Option<&'a str>,
        // subject: Option<Sub>,
        issuer: Option<Iss>,
    ) -> Self {
        ProtoPackageVerifier {
            signature,
            secret,
            // subject,
            issuer,
        }
    }

    pub fn verify(self, data: &[u8]) -> Request {
        if self.signature.is_none() {
            panic!("missing signature to compare");
        }

        if self.secret.is_none() {
            panic!("missing secret");
        }

        if !verify_signature(
            data,
            self.signature.unwrap(),
            self.secret.unwrap().as_bytes(),
        ) {
            panic!("Invalid signature!");
        }

        let decoded = match Request::decode(&mut &data[..]) {
            Ok(d) => d,
            Err(_) => panic!("error during decoding"),
        };

        if let Some(claims) = &decoded.claims {
            if claims.sub == -1 {
                panic!("no subject");
            }

            if let Some(issuer) = self.issuer {
                if issuer != claims.iss() {
                    panic!("invalid issuer");
                }
            } else {
                panic!("missing issuer");
            }

            let now = chrono::Utc::now().timestamp() as u64;
            if now > claims.exp {
                panic!("claims expired");
            }
        }

        decoded
    }

    pub fn builder() -> ProtoPackageVerifierBuilder<'a> {
        ProtoPackageVerifierBuilder::new()
    }
}

struct ProtoPackageVerifierBuilder<'a> {
    signature: Option<&'a str>,
    secret: Option<&'a str>,
    // subject: Option<Sub>,
    issuer: Option<Iss>,
}

impl<'a> ProtoPackageVerifierBuilder<'a> {
    fn new() -> Self {
        ProtoPackageVerifierBuilder {
            signature: None,
            secret: None,
            // subject: None,
            issuer: None,
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

    pub fn build(self) -> ProtoPackageVerifier<'a> {
        ProtoPackageVerifier::new(self.signature, self.secret, self.issuer)
    }
}

fn verify_signature(data: &[u8], signature: &str, secret: &[u8]) -> bool {
    signature == generate_signature(data, secret)
}

fn generate_signature(data: &[u8], secret: &[u8]) -> String {
    let mut mac = match Hmac::<Sha256>::new_from_slice(&secret) {
        Ok(m) => m,
        Err(_) => panic!("ERROR DURING SIGNING"),
    };

    mac.update(&data);
    let result = mac.finalize();
    let result_bytes = result.into_bytes();

    base16ct::lower::encode_string(&result_bytes)
}

fn generate_claims(data: Dat, subject: Sub) -> Claims {
    Claims {
        iss: Iss::Server.into(),
        sub: subject.into(),
        dat: Some(data),
        iat: chrono::Utc::now().timestamp() as u64,
        exp: (chrono::Utc::now() + chrono::Duration::seconds(30)).timestamp() as u64,
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

pub fn decode_proto(proto_bytes: &[u8], secret: &str, signature: &str) -> Request {
    let proto_package_verifier = ProtoPackageVerifier::builder()
        .with_signature(signature)
        .with_secret(secret)
        .with_issuer(Iss::Client)
        .build();

    proto_package_verifier.verify(proto_bytes)
}
