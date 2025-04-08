#[cfg(test)]
pub mod web_util {
    use mockall::predicate;

    use crate::{
        core::serf_proto::{claims::Dat, Claims, Iss, MigrationRequest, QueryRequest, Sub},
        web::{
            proto::ProtoPackage,
            util::{get_proto_package_result, MockRequestHandler},
        },
    };

    #[tokio::test]
    async fn test_get_proto_package_result_calls_handle_fetch() {
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
    async fn test_get_proto_package_result_calls_handle_mutate() {
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
    async fn test_get_proto_package_result_calls_handle_migrate() {
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
}
