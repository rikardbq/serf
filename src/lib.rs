pub mod cli;
pub mod core;
pub mod web;

#[allow(non_snake_case)]
#[cfg(test)]
pub mod web_util {
    use actix_web::http::header::{HeaderMap, HeaderName, HeaderValue};
    use mockall::predicate;

    use crate::{
        core::{
            error::{HeaderMalformedError, HeaderMissingError, SerfError, UndefinedError},
            serf_proto::{claims::Dat, Claims, Iss, MigrationRequest, QueryRequest, Sub},
        },
        web::{
            proto::ProtoPackage,
            util::{get_header_value, get_proto_package_result, MockRequestHandler},
        },
    };

    #[tokio::test]
    async fn test_get_proto_package_result__calls_handle_fetch() {
        let mut mock_handler = MockRequestHandler::new();
        let expected_proto_package = ProtoPackage {
            data: vec![1, 2, 3],
            signature: "any".to_string(),
        };

        mock_handler
            .expect_handle_fetch()
            .times(1)
            .with(predicate::always())
            .with(predicate::always())
            .returning(|_| {
                let res = Ok(ProtoPackage {
                    data: vec![1, 2, 3],
                    signature: "any".to_string(),
                });

                Box::pin(async move { res })
            });

        let claims = Claims {
            iss: Iss::Client.into(),
            iat: 1,
            exp: 2,
            sub: Sub::Fetch.into(),
            dat: Some(Dat::QueryRequest(QueryRequest::default())),
        };

        let result = get_proto_package_result(claims, &mock_handler).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_proto_package);
    }

    #[tokio::test]
    async fn test_get_proto_package_result__calls_handle_mutate() {
        let mut mock_handler = MockRequestHandler::new();
        let expected_proto_package = ProtoPackage {
            data: vec![1, 2, 3],
            signature: "any".to_string(),
        };

        mock_handler
            .expect_handle_mutate()
            .times(1)
            .with(predicate::always())
            .with(predicate::always())
            .returning(|_| {
                let res = Ok(ProtoPackage {
                    data: vec![1, 2, 3],
                    signature: "any".to_string(),
                });

                Box::pin(async move { res })
            });

        let claims = Claims {
            iss: Iss::Client.into(),
            iat: 1,
            exp: 2,
            sub: Sub::Mutate.into(),
            dat: Some(Dat::QueryRequest(QueryRequest::default())),
        };

        let result = get_proto_package_result(claims, &mock_handler).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_proto_package);
    }

    #[tokio::test]
    async fn test_get_proto_package_result__calls_handle_migrate() {
        let mut mock_handler = MockRequestHandler::new();
        let expected_proto_package = ProtoPackage {
            data: vec![1, 2, 3],
            signature: "any".to_string(),
        };

        mock_handler
            .expect_handle_migrate()
            .times(1)
            .with(predicate::always())
            .with(predicate::always())
            .returning(|_| {
                let res = Ok(ProtoPackage {
                    data: vec![1, 2, 3],
                    signature: "any".to_string(),
                });

                Box::pin(async move { res })
            });

        let claims = Claims {
            iss: Iss::Client.into(),
            iat: 1,
            exp: 2,
            sub: Sub::Migrate.into(),
            dat: Some(Dat::MigrationRequest(MigrationRequest::default())),
        };

        let result = get_proto_package_result(claims, &mock_handler).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_proto_package);
    }

    #[tokio::test]
    async fn test_get_proto_package_result__query_request_handle_incorrect_subject() {
        let mut mock_handler = MockRequestHandler::new();

        mock_handler.expect_handle_fetch().times(0);
        mock_handler.expect_handle_mutate().times(0);

        let claims = Claims {
            iss: Iss::Client.into(),
            iat: 1,
            exp: 2,
            sub: Sub::Migrate.into(),
            dat: Some(Dat::QueryRequest(QueryRequest::default())),
        };

        let result = get_proto_package_result(claims, &mock_handler).await;

        assert!(result.is_err());
        assert_eq!(
            result.expect_err("Should be UndefinedError"),
            UndefinedError::default()
        );
    }

    #[tokio::test]
    async fn test_get_proto_package_result__migration_request_handle_incorrect_subject() {
        let mut mock_handler = MockRequestHandler::new();

        mock_handler.expect_handle_migrate().times(0);

        let claims = Claims {
            iss: Iss::Client.into(),
            iat: 1,
            exp: 2,
            sub: Sub::Fetch.into(),
            dat: Some(Dat::MigrationRequest(MigrationRequest::default())),
        };

        let result = get_proto_package_result(claims, &mock_handler).await;

        assert!(result.is_err());
        assert_eq!(
            result.expect_err("Should be UndefinedError"),
            UndefinedError::default()
        );
    }

    #[test]
    fn test_header_value__get_success() {
        let mut headers = HeaderMap::new();
        headers.append(
            HeaderName::from_static("test"),
            HeaderValue::from_str("test_val").unwrap(),
        );

        let header_val = get_header_value(headers.get("test"));
        let expected_header_val = "test_val";

        assert!(header_val.is_ok());
        assert_eq!(header_val.unwrap(), expected_header_val);
    }

    #[test]
    fn test_header_value__get_header_missing_error() {
        let mut headers = HeaderMap::new();
        headers.append(
            HeaderName::from_static("test"),
            HeaderValue::from_str("test_val").unwrap(),
        );

        let header_val = get_header_value(headers.get("not_test"));

        assert!(header_val.is_err());
        assert_eq!(
            header_val.expect_err("Should be HeaderMissingError"),
            HeaderMissingError::default()
        );
    }

    #[test]
    fn test_header_value__get_header_malformed_error() {
        let mut headers = HeaderMap::new();
        headers.append(
            HeaderName::from_static("test"),
            HeaderValue::from_str("malförmed").unwrap(),
        );

        let header_val = get_header_value(headers.get("test"));

        assert!(header_val.is_err());
        assert_eq!(
            header_val.expect_err("Should be HeaderMalformedError"),
            HeaderMalformedError::default()
        );
    }
}

#[allow(non_snake_case)]
#[cfg(test)]
pub mod web_proto {
    use std::any::{Any, TypeId};

    use actix_web::http::header::{HeaderMap, HeaderName, HeaderValue};
    use hmac::digest::typenum::assert_type_eq;
    use mockall::predicate;
    use prost::Message;

    use crate::{
        core::{
            error::{HeaderMalformedError, HeaderMissingError, SerfError, UndefinedError},
            serf_proto::{claims::Dat, Claims, Iss, MigrationRequest, QueryRequest, Request, Sub},
        },
        web::{
            proto::{ProtoPackage, ProtoPackageVerifier, ProtoPackageVerifierBuilder},
            util::{get_header_value, get_proto_package_result, MockRequestHandler},
        },
    };

    #[test]
    fn test_proto_package_builder__build_proto_package() {
        let query_request_dat =
            QueryRequest::as_dat("SELECT * FROM test_data_table;".to_string(), vec![]);

        let expected_issuer = Iss::Server;
        let expected_subject = Sub::Fetch;

        let secret = "test_hash";
        let proto_package_res = ProtoPackage::builder()
            .with_data(query_request_dat.clone())
            .with_subject(Sub::Fetch)
            .sign(secret);

        assert!(proto_package_res.is_ok());

        let proto_package_data = proto_package_res.unwrap().data;
        let decoded_proto_package = Request::decode(&mut &proto_package_data[..]);
        assert!(decoded_proto_package.is_ok());

        let decoded_claims = decoded_proto_package.unwrap().claims;
        assert!(decoded_claims.is_some());

        let unwrapped_claims = decoded_claims.unwrap();
        assert_eq!(unwrapped_claims.iss(), expected_issuer);
        assert_eq!(unwrapped_claims.sub(), expected_subject);
        assert_eq!(unwrapped_claims.iat.type_id(), 0u64.type_id());
        assert_eq!(unwrapped_claims.exp.type_id(), 0u64.type_id());
        assert_eq!(unwrapped_claims.exp, unwrapped_claims.iat + 30);

        let decoded_dat = unwrapped_claims.dat;
        assert!(decoded_dat.is_some());
        assert_eq!(decoded_dat.unwrap(), query_request_dat);
    }

    #[test]
    fn test_proto_package_verifier_builder__verify_proto_package() {
        let expected_query_request_dat =
            QueryRequest::as_dat("SELECT * FROM test_data_table;".to_string(), vec![]);

        let expected_issuer = Iss::Server;
        let expected_subject = Sub::Fetch;
        let expected_signature = "b96deb8fed7f9c0335b7c5ffa65697e5042ef0af87a8494c284e234424a9bab9";

        let secret = "test_hash";
        let proto_package_data: [u8; 52] = [
            10, 50, 8, 1, 16, 1, 42, 32, 10, 30, 83, 69, 76, 69, 67, 84, 32, 42, 32, 70, 82, 79,
            77, 32, 116, 101, 115, 116, 95, 100, 97, 116, 97, 95, 116, 97, 98, 108, 101, 59, 64,
            235, 198, 219, 191, 6, 72, 137, 199, 219, 191, 6,
        ];
        let request_res = ProtoPackageVerifier::builder()
            .with_issuer(Iss::Server)
            .with_secret(secret)
            .with_signature(expected_signature)
            .build()
            .verify(&proto_package_data);

        assert!(request_res.is_ok());

        let request_claims = request_res.unwrap().claims.unwrap();
        assert_eq!(request_claims.iss(), expected_issuer);
        assert_eq!(request_claims.sub(), expected_subject);
        assert_eq!(request_claims.iat.type_id(), 0u64.type_id());
        assert_eq!(request_claims.exp.type_id(), 0u64.type_id());
        assert_eq!(request_claims.exp, request_claims.iat + 30);
        assert_eq!(request_claims.dat.unwrap(), expected_query_request_dat);
    }
}
