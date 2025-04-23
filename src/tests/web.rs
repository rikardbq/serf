#[allow(non_snake_case)]
pub mod util {
    use actix_web::http::header::{HeaderMap, HeaderName, HeaderValue};
    use mockall::predicate;

    use crate::{
        core::{
            error::{HeaderMalformedError, HeaderMissingError, SerfError, UndefinedError},
            serf_proto::{claims::Dat, Claims, Iss, MigrationRequest, QueryRequest, Sub},
        },
        web::{
            proto::ProtoPackage,
            util::{
                check_content_type, extract_headers, get_header_value, get_proto_package_result,
                MockRequestHandler,
            },
        },
    };

    #[tokio::test]
    async fn test_get_proto_package_result__calls_handle_fetch() {
        let expected_proto_package = ProtoPackage {
            data: vec![1, 2, 3],
            signature: "any".to_string(),
        };

        let mut mock_handler = MockRequestHandler::new();
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
        let expected_proto_package = ProtoPackage {
            data: vec![1, 2, 3],
            signature: "any".to_string(),
        };

        let mut mock_handler = MockRequestHandler::new();
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
        let expected_proto_package = ProtoPackage {
            data: vec![1, 2, 3],
            signature: "any".to_string(),
        };

        let mut mock_handler = MockRequestHandler::new();
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
        let expected_error = UndefinedError::default();

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
            expected_error
        );
    }

    #[tokio::test]
    async fn test_get_proto_package_result__migration_request_handle_incorrect_subject() {
        let expected_error = UndefinedError::default();

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
            expected_error
        );
    }

    #[test]
    fn test_header_value__get_success() {
        let expected_header_val = "test_val";

        let mut headers = HeaderMap::new();
        headers.append(
            HeaderName::from_static("test"),
            HeaderValue::from_str("test_val").unwrap(),
        );
        let header_val = get_header_value(headers.get("test"));

        assert!(header_val.is_ok());
        assert_eq!(header_val.unwrap(), expected_header_val);
    }

    #[test]
    fn test_header_value__get_header_missing_error() {
        let expected_error = HeaderMissingError::default();

        let mut headers = HeaderMap::new();
        headers.append(
            HeaderName::from_static("test"),
            HeaderValue::from_str("test_val").unwrap(),
        );
        let header_val = get_header_value(headers.get("not_test"));

        assert!(header_val.is_err());
        assert_eq!(
            header_val.expect_err("Should be HeaderMissingError"),
            expected_error
        );
    }

    #[test]
    fn test_header_value__get_header_malformed_error() {
        let expected_error = HeaderMalformedError::default();

        let mut headers = HeaderMap::new();
        headers.append(
            HeaderName::from_static("test"),
            HeaderValue::from_str("malförmed").unwrap(),
        );
        let header_val = get_header_value(headers.get("test"));

        assert!(header_val.is_err());
        assert_eq!(
            header_val.expect_err("Should be HeaderMalformedError"),
            expected_error
        );
    }

    #[test]
    fn test_extract_headers__extract_success() {
        let expected_headers_tuple = ("application/protobuf", "some_value_0", "some_value_1");

        let mut headers = HeaderMap::new();
        headers.append(
            HeaderName::from_static("0"),
            HeaderValue::from_str("some_value_0").unwrap(),
        );
        headers.append(
            HeaderName::from_static("1"),
            HeaderValue::from_str("some_value_1").unwrap(),
        );
        headers.append(
            HeaderName::from_static("content-type"),
            HeaderValue::from_str("application/protobuf").unwrap(),
        );
        let headers_res = extract_headers(&headers);

        assert!(headers_res.is_ok());
        assert_eq!(headers_res.unwrap(), expected_headers_tuple);
    }

    #[test]
    fn test_extract_headers__extract_missing_content_type() {
        let expected_error = HeaderMissingError::default();

        let mut headers = HeaderMap::new();
        headers.append(
            HeaderName::from_static("0"),
            HeaderValue::from_str("some_value_0").unwrap(),
        );
        headers.append(
            HeaderName::from_static("1"),
            HeaderValue::from_str("some_value_1").unwrap(),
        );

        let headers_res = extract_headers(&headers);

        assert!(headers_res.is_err());
        assert_eq!(
            headers_res.expect_err("Should be HeaderMissingError"),
            expected_error
        );
    }

    #[test]
    fn test_extract_headers__extract_missing_header_0() {
        let expected_error = HeaderMissingError::default();

        let mut headers = HeaderMap::new();
        headers.append(
            HeaderName::from_static("1"),
            HeaderValue::from_str("some_value_1").unwrap(),
        );
        headers.append(
            HeaderName::from_static("content-type"),
            HeaderValue::from_str("application/protobuf").unwrap(),
        );
        let headers_res = extract_headers(&headers);

        assert!(headers_res.is_err());
        assert_eq!(
            headers_res.expect_err("Should be HeaderMissingError"),
            expected_error
        );
    }

    #[test]
    fn test_extract_headers__extract_missing_header_1() {
        let expected_error = HeaderMissingError::default();

        let mut headers = HeaderMap::new();
        headers.append(
            HeaderName::from_static("0"),
            HeaderValue::from_str("some_value_0").unwrap(),
        );
        headers.append(
            HeaderName::from_static("content-type"),
            HeaderValue::from_str("application/protobuf").unwrap(),
        );
        let headers_res = extract_headers(&headers);

        assert!(headers_res.is_err());
        assert_eq!(
            headers_res.expect_err("Should be HeaderMissingError"),
            expected_error
        );
    }

    #[test]
    fn test_check_content_type__content_type_ok() {
        let check_content_type_res = check_content_type("application/protobuf");

        assert!(check_content_type_res.is_ok());
    }

    #[test]
    fn test_check_content_type__content_type_err() {
        let expected_error = HeaderMalformedError::with_message("Content-Type not supported");
        let check_content_type_res = check_content_type("application/json");

        assert!(check_content_type_res.is_err());
        assert_eq!(
            check_content_type_res.expect_err("Should be HeaderMalformedError"),
            expected_error
        );
    }
}

#[allow(non_snake_case)]
#[cfg(test)]
pub mod proto {
    use std::any::Any;

    use prost::Message;

    use crate::{
        core::{
            error::ProtoPackageError,
            serf_proto::{Claims, Iss, QueryRequest, Request, Sub},
        },
        tests::test_utils::constants::TEST_NOW_TIMESTAMP,
        web::proto::{generate_signature, ProtoPackage, ProtoPackageVerifier},
    };

    #[test]
    fn test_proto_package_builder__build_proto_package_success() {
        let expected_issuer = Iss::Server;
        let expected_subject = Sub::Fetch;
        let query_request_dat =
            QueryRequest::as_dat("SELECT * FROM test_data_table;".to_string(), vec![]);

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
    fn test_proto_package_builder__build_proto_package_fail_missing_subject() {
        let expected_error = ProtoPackageError::signing_error("missing subject");
        let query_request_dat =
            QueryRequest::as_dat("SELECT * FROM test_data_table;".to_string(), vec![]);

        let secret = "test_hash";
        let proto_package_res = ProtoPackage::builder()
            .with_data(query_request_dat.clone())
            .sign(secret);

        assert!(proto_package_res.is_err());
        assert_eq!(
            proto_package_res.expect_err("Should be ProtoPackageError::SIGN"),
            expected_error
        );
    }

    #[test]
    fn test_proto_package_builder__build_proto_package_fail_missing_data() {
        let expected_error = ProtoPackageError::signing_error("missing data");

        let secret = "test_hash";
        let subject = Sub::Fetch;
        let proto_package_res = ProtoPackage::builder().with_subject(subject).sign(secret);

        assert!(proto_package_res.is_err());
        assert_eq!(
            proto_package_res.expect_err("Should be ProtoPackageError::SIGN"),
            expected_error
        );
    }

    #[test]
    fn test_proto_package_verifier__verify_proto_package_success() {
        let expected_issuer = Iss::Server;
        let expected_subject = Sub::Fetch;
        let expected_query_request_dat =
            QueryRequest::as_dat("SELECT * FROM test_data_table;".to_string(), vec![]);

        let proto_package_data: [u8; 52] = [
            10, 50, 8, 1, 16, 1, 42, 32, 10, 30, 83, 69, 76, 69, 67, 84, 32, 42, 32, 70, 82, 79,
            77, 32, 116, 101, 115, 116, 95, 100, 97, 116, 97, 95, 116, 97, 98, 108, 101, 59, 64,
            235, 198, 219, 191, 6, 72, 137, 199, 219, 191, 6,
        ];

        let secret = "test_hash";
        let issuer = Iss::Server;
        let signature = "b96deb8fed7f9c0335b7c5ffa65697e5042ef0af87a8494c284e234424a9bab9";
        let request_res = ProtoPackageVerifier::builder()
            .with_issuer(issuer)
            .with_secret(secret)
            .with_signature(signature)
            .TEST_with_now_timestamp(TEST_NOW_TIMESTAMP)
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

    #[test]
    fn test_proto_package_verifier__verify_proto_package_fail_invalid_signature() {
        let expected_error = ProtoPackageError::verification_error("invalid signature");

        let proto_package_data: [u8; 52] = [
            10, 50, 8, 1, 16, 1, 42, 32, 10, 30, 83, 69, 76, 69, 67, 84, 32, 42, 32, 70, 82, 79,
            77, 32, 116, 101, 115, 116, 95, 100, 97, 116, 97, 95, 116, 97, 98, 108, 101, 59, 64,
            235, 198, 219, 191, 6, 72, 137, 199, 219, 191, 6,
        ];

        let secret = "test_hash";
        let issuer = Iss::Server;
        let signature = "c96deb8fed7f9c0335b7c5ffa65697e5042ef0af87a8494c284e234424a9bab9";
        let request_res = ProtoPackageVerifier::builder()
            .with_issuer(issuer)
            .with_secret(secret)
            .with_signature(signature)
            .TEST_with_now_timestamp(TEST_NOW_TIMESTAMP)
            .build()
            .verify(&proto_package_data);

        assert!(request_res.is_err());
        assert_eq!(
            request_res.expect_err("Should be ProtoPackageError::VERIFY"),
            expected_error
        );
    }

    #[test]
    fn test_proto_package_verifier__verify_proto_package_fail_missing_signature() {
        let expected_error = ProtoPackageError::verification_error("missing signature");

        let proto_package_data: [u8; 52] = [
            10, 50, 8, 1, 16, 1, 42, 32, 10, 30, 83, 69, 76, 69, 67, 84, 32, 42, 32, 70, 82, 79,
            77, 32, 116, 101, 115, 116, 95, 100, 97, 116, 97, 95, 116, 97, 98, 108, 101, 59, 64,
            235, 198, 219, 191, 6, 72, 137, 199, 219, 191, 6,
        ];

        let secret = "test_hash";
        let issuer = Iss::Server;
        let request_res = ProtoPackageVerifier::builder()
            .with_issuer(issuer)
            .with_secret(secret)
            .TEST_with_now_timestamp(TEST_NOW_TIMESTAMP)
            .build()
            .verify(&proto_package_data);

        assert!(request_res.is_err());
        assert_eq!(
            request_res.expect_err("Should be ProtoPackagerError::VERIFY"),
            expected_error
        );
    }

    #[test]
    fn test_proto_package_verifier__verify_proto_package_fail_missing_secret() {
        let expected_error = ProtoPackageError::verification_error("missing secret");

        let proto_package_data: [u8; 52] = [
            10, 50, 8, 1, 16, 1, 42, 32, 10, 30, 83, 69, 76, 69, 67, 84, 32, 42, 32, 70, 82, 79,
            77, 32, 116, 101, 115, 116, 95, 100, 97, 116, 97, 95, 116, 97, 98, 108, 101, 59, 64,
            235, 198, 219, 191, 6, 72, 137, 199, 219, 191, 6,
        ];

        let issuer = Iss::Server;
        let signature = "b96deb8fed7f9c0335b7c5ffa65697e5042ef0af87a8494c284e234424a9bab9";
        let request_res = ProtoPackageVerifier::builder()
            .with_issuer(issuer)
            .with_signature(signature)
            .TEST_with_now_timestamp(TEST_NOW_TIMESTAMP)
            .build()
            .verify(&proto_package_data);

        assert!(request_res.is_err());
        assert_eq!(
            request_res.expect_err("Should be ProtoPackagerError::VERIFY"),
            expected_error
        );
    }

    #[test]
    fn test_proto_package_verifier__verify_proto_package_fail_invalid_claims_subject() {
        let expected_error = ProtoPackageError::verification_error("invalid claims subject");

        let query_request_dat =
            QueryRequest::as_dat("SELECT * FROM test_data_table;".to_string(), vec![]);

        let claims_subject = -1;
        let claims_issuer = Iss::Server;
        let claims = Claims {
            iss: claims_issuer.into(),
            sub: claims_subject,
            dat: Some(query_request_dat),
            iat: TEST_NOW_TIMESTAMP,
            exp: TEST_NOW_TIMESTAMP + 30,
        };
        let request = Request {
            claims: Some(claims),
            error: None,
        };

        let mut buf = Vec::new();
        buf.reserve(request.encoded_len());

        let encode_res = request.encode(&mut buf);
        assert!(encode_res.is_ok());

        let secret = "test_hash";
        let issuer = Iss::Server;
        let signature = "e6e2506fe1721756369a852aad91da065602585733d911001dea60405e689fce";
        let request_res = ProtoPackageVerifier::builder()
            .with_issuer(issuer)
            .with_secret(secret)
            .with_signature(&signature)
            .TEST_with_now_timestamp(TEST_NOW_TIMESTAMP)
            .build()
            .verify(&buf);

        assert!(request_res.is_err());
        assert_eq!(
            request_res.expect_err("Should be ProtoPackageError::VERIFY"),
            expected_error
        );
    }

    #[test]
    fn test_proto_package_verifier__verify_proto_package_fail_invalid_claims_issuer() {
        let expected_error = ProtoPackageError::verification_error("invalid claims issuer");

        let query_request_dat =
            QueryRequest::as_dat("SELECT * FROM test_data_table;".to_string(), vec![]);

        let claims_subject = Sub::Fetch;
        let claims_issuer = Iss::Client;
        let claims = Claims {
            iss: claims_issuer.into(),
            sub: claims_subject.into(),
            dat: Some(query_request_dat),
            iat: TEST_NOW_TIMESTAMP,
            exp: TEST_NOW_TIMESTAMP + 30,
        };
        let request = Request {
            claims: Some(claims),
            error: None,
        };

        let mut buf = Vec::new();
        buf.reserve(request.encoded_len());

        let encode_res = request.encode(&mut buf);
        assert!(encode_res.is_ok());

        let secret = "test_hash";
        let issuer = Iss::Server;
        let signature = "e89c1194876ab3d2432362bd10192698eda080bab128f1759cd08e00bcd025b0";
        let request_res = ProtoPackageVerifier::builder()
            .with_issuer(issuer)
            .with_secret(secret)
            .with_signature(&signature)
            .TEST_with_now_timestamp(TEST_NOW_TIMESTAMP)
            .build()
            .verify(&buf);

        assert!(request_res.is_err());
        assert_eq!(
            request_res.expect_err("Should be ProtoPackageError::VERIFY"),
            expected_error
        );
    }

    #[test]
    fn test_proto_package_verifier__verify_proto_package_fail_missing_claims_data() {
        let expected_error = ProtoPackageError::verification_error("missing claims data");

        let claims_subject = Sub::Fetch;
        let claims_issuer = Iss::Server;
        let claims = Claims {
            iss: claims_issuer.into(),
            sub: claims_subject.into(),
            dat: None,
            iat: TEST_NOW_TIMESTAMP,
            exp: TEST_NOW_TIMESTAMP + 30,
        };
        let request = Request {
            claims: Some(claims),
            error: None,
        };

        let mut buf = Vec::new();
        buf.reserve(request.encoded_len());

        let encode_res = request.encode(&mut buf);
        assert!(encode_res.is_ok());

        let secret = "test_hash";
        let issuer = Iss::Server;
        let signature = "55675c8d58c09f486b079e9136e3039014e3725d457fdc5f0992dc7ca99cf1f0";
        let request_res = ProtoPackageVerifier::builder()
            .with_issuer(issuer)
            .with_secret(secret)
            .with_signature(&signature)
            .TEST_with_now_timestamp(TEST_NOW_TIMESTAMP)
            .build()
            .verify(&buf);

        assert!(request_res.is_err());
        assert_eq!(
            request_res.expect_err("Should be ProtoPackageError::VERIFY"),
            expected_error
        );
    }

    #[test]
    fn test_proto_package_verifier__verify_proto_package_fail_claims_expired() {
        let expected_error = ProtoPackageError::verification_error("claims expired");

        let query_request_dat =
            QueryRequest::as_dat("SELECT * FROM test_data_table;".to_string(), vec![]);

        let claims_subject = Sub::Fetch;
        let claims_issuer = Iss::Server;
        let claims = Claims {
            iss: claims_issuer.into(),
            sub: claims_subject.into(),
            dat: Some(query_request_dat),
            iat: TEST_NOW_TIMESTAMP - 60,
            exp: TEST_NOW_TIMESTAMP - 30,
        };
        let request = Request {
            claims: Some(claims),
            error: None,
        };

        let mut buf = Vec::new();
        buf.reserve(request.encoded_len());

        let encode_res = request.encode(&mut buf);
        assert!(encode_res.is_ok());

        let secret = "test_hash";
        let issuer = Iss::Server;
        let signature = "0e3477efad0e21df052c1a131ef9960158b22da7c156d3aa955c1ae19dd869f9";
        let request_res = ProtoPackageVerifier::builder()
            .with_issuer(issuer)
            .with_secret(secret)
            .with_signature(&signature)
            .TEST_with_now_timestamp(TEST_NOW_TIMESTAMP)
            .build()
            .verify(&buf);

        assert!(request_res.is_err());
        assert_eq!(
            request_res.expect_err("Should be ProtoPackageError::VERIFY"),
            expected_error
        );
    }

    #[test]
    fn test_proto_package_verifier__verify_proto_package_fail_missing_claims() {
        let expected_error = ProtoPackageError::verification_error("missing claims");

        let request = Request {
            claims: None,
            error: None,
        };

        let mut buf = Vec::new();
        buf.reserve(request.encoded_len());

        let encode_res = request.encode(&mut buf);
        assert!(encode_res.is_ok());

        let secret = "test_hash";
        let issuer = Iss::Server;
        println!("{}", generate_signature(&buf, secret.as_bytes()));
        let signature = "70a9b9745e67688e57fc4aa03cbc514a8ea615367fc7622155d147afd1f1fcd3";
        let request_res = ProtoPackageVerifier::builder()
            .with_issuer(issuer)
            .with_secret(secret)
            .with_signature(&signature)
            .build()
            .verify(&buf);

        assert!(request_res.is_err());
        assert_eq!(
            request_res.expect_err("Should be ProtoPackageError::VERIFY"),
            expected_error
        );
    }

    #[test]
    fn test_generate_signature__ensure_same() {
        // HMAC<SHA256> with secret: "test_secret"
        let expected_signature = "ae02cd09103fd99c64802e1be1a50376d802a3ae99caf49c1cdd4ab6b6ee050f";

        let data: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
        let secret = "test_secret";
        let signature1 = generate_signature(&data, secret.as_bytes());
        let signature2 = generate_signature(&data, secret.as_bytes());

        assert_eq!(signature1, expected_signature);
        assert_eq!(signature2, expected_signature);
    }
}
