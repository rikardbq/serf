// INTEGRATION TESTS

#[allow(non_snake_case)]
#[cfg(test)]
pub mod util {
    use serf::{
        core::{
            error::{DatabaseError, SerfError, UserNotAllowedError},
            serf_proto::{
                query_arg, Claims, FetchResponse, Iss, MigrationRequest, MigrationResponse,
                MutationResponse, QueryArg, QueryRequest, Sub,
            },
        },
        web::{
            proto::{encode_proto, ProtoPackage},
            util::{get_proto_package_result, ProtoPackageResultHandler},
        },
    };

    use serde_json::json;
    use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

    async fn setup_test_db() -> SqlitePool {
        let db = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("Failed to create in-memory SQLite pool");

        sqlx::query(
            r#"
            CREATE TABLE test_data_table (
                id INTEGER PRIMARY KEY NOT NULL,
                im_data	TEXT,
                im_data_too TEXT,
                im_data_aswell INTEGER NOT NULL
            );
        "#,
        )
        .execute(&db)
        .await
        .expect("Failed to create test data table");

        sqlx::query(
            r#"
            CREATE TABLE strict_test_data_table (
                id INTEGER PRIMARY KEY NOT NULL,
                im_data	TEXT,
                im_data_too TEXT,
                im_data_aswell INTEGER NOT NULL
            ) STRICT;
        "#,
        )
        .execute(&db)
        .await
        .expect("Failed to create strict test data table");

        sqlx::query(
            r#"
            INSERT INTO test_data_table(im_data, im_data_too, im_data_aswell) VALUES(?, ?, ?);
            "#,
        )
        .bind("test_value1")
        .bind("test_value2")
        .bind(123)
        .execute(&db)
        .await
        .expect("Failed to add test entry to test data table");

        db
    }

    // USER ACCESS LEVEL
    #[tokio::test]
    async fn test_handle_mutate__user_access_too_low() {
        let db = setup_test_db().await;
        let result_handler = ProtoPackageResultHandler::new(1, "test_hash", &db);

        let query_request_dat = QueryRequest::as_dat(
            "INSERT INTO test_data_table(im_data, im_data_too) VALUES(?, ?);".to_string(),
            vec![
                QueryArg::new(query_arg::Value::String("test_data".to_string())),
                QueryArg::new(query_arg::Value::String("test_data_too".to_string())),
            ],
        );

        let claims = Claims {
            iss: Iss::Client.into(),
            iat: chrono::Utc::now().timestamp() as u64,
            exp: (chrono::Utc::now() + chrono::Duration::seconds(30)).timestamp() as u64,
            sub: Sub::Mutate.into(),
            dat: Some(query_request_dat),
        };

        let result = get_proto_package_result(claims, &result_handler).await;

        assert!(result.is_err());
        assert_eq!(
            result.expect_err("Should be UserNotAllowedError"),
            UserNotAllowedError::default()
        );
    }

    #[tokio::test]
    async fn test_handle_fetch__user_access_too_low() {
        let db = setup_test_db().await;
        let result_handler = ProtoPackageResultHandler::new(0, "test_hash", &db);

        let query_request_dat = QueryRequest::as_dat(
            "SELECT * FROM test_data_table;".to_string(),
            vec![
                // QueryArg::new(query_arg::Value::Int(1))
            ],
        );

        let claims = Claims {
            iss: Iss::Client.into(),
            iat: chrono::Utc::now().timestamp() as u64,
            exp: (chrono::Utc::now() + chrono::Duration::seconds(30)).timestamp() as u64,
            sub: Sub::Fetch.into(),
            dat: Some(query_request_dat),
        };

        let result = get_proto_package_result(claims, &result_handler).await;

        assert!(result.is_err());
        assert_eq!(
            result.expect_err("Should be UserNotAllowedError"),
            UserNotAllowedError::default()
        );
    }

    #[tokio::test]
    async fn test_handle_migrate__user_access_too_low() {
        let db = setup_test_db().await;
        let result_handler = ProtoPackageResultHandler::new(1, "test_hash", &db);

        let migration_request_dat = MigrationRequest::as_dat(
            "1__add_test_col".to_string(),
            "ALTER TABLE test_data_table ADD COLUMN test_col TEXT;".to_string(),
        );

        let claims = Claims {
            iss: Iss::Client.into(),
            iat: chrono::Utc::now().timestamp() as u64,
            exp: (chrono::Utc::now() + chrono::Duration::seconds(30)).timestamp() as u64,
            sub: Sub::Migrate.into(),
            dat: Some(migration_request_dat),
        };

        let result = get_proto_package_result(claims, &result_handler).await;

        assert!(result.is_err());
        assert_eq!(
            result.expect_err("Should be UserNotAllowedError"),
            UserNotAllowedError::default()
        );
    }
    // USER ACCESS LEVEL END

    // MIGRATE
    #[tokio::test]
    async fn test_handle_migrate__migration_success() {
        let db = setup_test_db().await;
        let username_password_hash = "test_hash";
        let result_handler = ProtoPackageResultHandler::new(2, username_password_hash, &db);
        let expected_result_proto_package = encode_proto(
            MigrationResponse::as_dat(true),
            Sub::Data,
            username_password_hash,
        );

        let migration_request_dat = MigrationRequest::as_dat(
            "1__add_test_col".to_string(),
            "ALTER TABLE test_data_table ADD COLUMN test_col TEXT;".to_string(),
        );

        let claims = Claims {
            iss: Iss::Client.into(),
            iat: chrono::Utc::now().timestamp() as u64,
            exp: (chrono::Utc::now() + chrono::Duration::seconds(30)).timestamp() as u64,
            sub: Sub::Migrate.into(),
            dat: Some(migration_request_dat),
        };

        let result = get_proto_package_result(claims, &result_handler).await;
        let db_content = sqlx::query("SELECT test_col FROM test_data_table;")
            .execute(&db)
            .await;
        let migration_table_content = sqlx::query("SELECT * FROM __migrations_tracker_t__;")
            .fetch_all(&db)
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_result_proto_package.unwrap());
        assert!(db_content.is_ok());
        assert!(migration_table_content.is_ok());
        assert_eq!(migration_table_content.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_handle_migrate__migration_fail_migration_already_exists() {
        let db = setup_test_db().await;
        let username_password_hash = "test_hash";
        let result_handler = ProtoPackageResultHandler::new(2, username_password_hash, &db);
        let now = chrono::Utc::now().timestamp() as u64;

        let expected_result_proto_package = ProtoPackage::builder()
            .with_data(MigrationResponse::as_dat(true))
            .with_subject(Sub::Data)
            .with_iat(now)
            .sign(username_password_hash);

        let migration_request_dat_1 = MigrationRequest::as_dat(
            "1__add_test_col".to_string(),
            "ALTER TABLE test_data_table ADD COLUMN test_col TEXT;".to_string(),
        );

        let claims_1 = Claims {
            iss: Iss::Client.into(),
            iat: now,
            exp: now + 30,
            sub: Sub::Migrate.into(),
            dat: Some(migration_request_dat_1),
        };

        let migration_request_dat_2 = MigrationRequest::as_dat(
            "1__add_test_col".to_string(),
            "ALTER TABLE test_data_table ADD COLUMN test_col_2 TEXT;".to_string(),
        );

        let claims_2 = Claims {
            iss: Iss::Client.into(),
            iat: chrono::Utc::now().timestamp() as u64,
            exp: (chrono::Utc::now() + chrono::Duration::seconds(30)).timestamp() as u64,
            sub: Sub::Migrate.into(),
            dat: Some(migration_request_dat_2),
        };

        let result_1 = get_proto_package_result(claims_1, &result_handler).await;
        let db_content_1 = sqlx::query("SELECT test_col FROM test_data_table;")
            .execute(&db)
            .await;
        let migration_table_content_1 = sqlx::query("SELECT * FROM __migrations_tracker_t__;")
            .fetch_all(&db)
            .await;

        let result_2 = get_proto_package_result(claims_2, &result_handler).await;
        let db_content_2 = sqlx::query("SELECT test_col FROM test_data_table;")
            .execute(&db)
            .await;
        let migration_table_content_2 = sqlx::query("SELECT * FROM __migrations_tracker_t__;")
            .fetch_all(&db)
            .await;

        assert!(result_1.is_ok());
        assert_eq!(result_1.unwrap(), expected_result_proto_package.unwrap());
        assert!(db_content_1.is_ok());
        assert!(migration_table_content_1.is_ok());
        assert_eq!(migration_table_content_1.unwrap().len(), 1);

        assert!(result_2.is_err());
        assert_eq!(
            result_2.expect_err("Should be DatabaseError"),
            DatabaseError::with_message("UNIQUE constraint failed: __migrations_tracker_t__.name")
        );
        assert!(db_content_2.is_ok());
        assert!(migration_table_content_2.is_ok());
        assert_eq!(migration_table_content_2.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_handle_migrate__migration_fail_column_already_exists() {
        let db = setup_test_db().await;
        let username_password_hash = "test_hash";
        let result_handler = ProtoPackageResultHandler::new(2, username_password_hash, &db);
        let now = chrono::Utc::now().timestamp() as u64;

        let expected_result_proto_package_1 = ProtoPackage::builder()
            .with_data(MigrationResponse::as_dat(true))
            .with_subject(Sub::Data)
            .with_iat(now)
            .sign(username_password_hash);

        let expected_result_proto_package_2 = ProtoPackage::builder()
            .with_data(MigrationResponse::as_dat(false))
            .with_subject(Sub::Data)
            .with_iat(now)
            .sign(username_password_hash);

        let migration_request_dat_1 = MigrationRequest::as_dat(
            "1__add_test_col".to_string(),
            "ALTER TABLE test_data_table ADD COLUMN test_col TEXT;".to_string(),
        );

        let claims_1 = Claims {
            iss: Iss::Client.into(),
            iat: now,
            exp: now + 30,
            sub: Sub::Migrate.into(),
            dat: Some(migration_request_dat_1),
        };

        let migration_request_dat_2 = MigrationRequest::as_dat(
            "2__add_test_col_again".to_string(),
            "ALTER TABLE test_data_table ADD COLUMN test_col TEXT;".to_string(),
        );

        let claims_2 = Claims {
            iss: Iss::Client.into(),
            iat: now,
            exp: now + 30,
            sub: Sub::Migrate.into(),
            dat: Some(migration_request_dat_2),
        };

        let result_1 = get_proto_package_result(claims_1, &result_handler).await;
        let db_content_1 = sqlx::query("SELECT test_col FROM test_data_table;")
            .execute(&db)
            .await;
        let migration_table_content_1 = sqlx::query("SELECT * FROM __migrations_tracker_t__;")
            .fetch_all(&db)
            .await;

        let result_2 = get_proto_package_result(claims_2, &result_handler).await;
        let db_content_2 = sqlx::query("SELECT test_col FROM test_data_table;")
            .execute(&db)
            .await;
        let migration_table_content_2 = sqlx::query("SELECT * FROM __migrations_tracker_t__;")
            .fetch_all(&db)
            .await;

        assert!(result_1.is_ok());
        assert_eq!(result_1.unwrap(), expected_result_proto_package_1.unwrap());
        assert!(db_content_1.is_ok());
        assert!(migration_table_content_1.is_ok());
        assert_eq!(migration_table_content_1.unwrap().len(), 1);

        assert!(result_2.is_ok());
        assert_eq!(result_2.unwrap(), expected_result_proto_package_2.unwrap());
        assert!(db_content_2.is_ok());
        assert!(migration_table_content_2.is_ok());
        assert_eq!(migration_table_content_2.unwrap().len(), 1);
    }
    // MIGRATE

    // FETCH
    #[tokio::test]
    async fn test_handle_fetch__fetch_data_success() {
        let db = setup_test_db().await;
        let username_password_hash = "test_hash";
        let result_handler = ProtoPackageResultHandler::new(1, username_password_hash, &db);
        let now = chrono::Utc::now().timestamp() as u64;
        let expected_result_json = json!([
            {
                "id": 1,
                "im_data": "test_value1",
                "im_data_too": "test_value2",
                "im_data_aswell": 123
            }
        ]);

        let expected_result_proto_package = ProtoPackage::builder()
            .with_data(FetchResponse::as_dat(
                serde_json::to_vec(&expected_result_json).unwrap(),
            ))
            .with_subject(Sub::Data)
            .with_iat(now)
            .sign(username_password_hash);

        let query_request_dat = QueryRequest::as_dat(
            "SELECT * FROM test_data_table WHERE id = ?;".to_string(),
            vec![QueryArg::new(query_arg::Value::Int(1))],
        );

        let claims = Claims {
            iss: Iss::Client.into(),
            iat: now,
            exp: now + 30,
            sub: Sub::Fetch.into(),
            dat: Some(query_request_dat),
        };

        let result = get_proto_package_result(claims, &result_handler).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_result_proto_package.unwrap());
    }

    #[tokio::test]
    async fn test_handle_fetch__fetch_data_fail() {
        let db = setup_test_db().await;
        let username_password_hash = "test_hash";
        let result_handler = ProtoPackageResultHandler::new(1, username_password_hash, &db);

        let query_request_dat = QueryRequest::as_dat(
            "SELECT * FROM test_data_table WHERE non_existing_col = ?;".to_string(),
            vec![QueryArg::new(query_arg::Value::Int(1))],
        );

        let claims = Claims {
            iss: Iss::Client.into(),
            iat: chrono::Utc::now().timestamp() as u64,
            exp: (chrono::Utc::now() + chrono::Duration::seconds(30)).timestamp() as u64,
            sub: Sub::Fetch.into(),
            dat: Some(query_request_dat),
        };

        let result = get_proto_package_result(claims, &result_handler).await;

        assert!(result.is_err());
        assert_eq!(
            result.expect_err("Should be DatabaseError"),
            DatabaseError::with_message("no such column: non_existing_col")
        );
    }
    // FETCH END

    // MUTATE
    #[tokio::test]
    async fn test_handle_mutate__mutate_data_update_entry_success() {
        let db = setup_test_db().await;
        let username_password_hash = "test_hash";
        let result_handler = ProtoPackageResultHandler::new(2, username_password_hash, &db);
        let now = chrono::Utc::now().timestamp() as u64;

        let expected_result_proto_package = ProtoPackage::builder()
            .with_data(MutationResponse::as_dat(1, 1))
            .with_subject(Sub::Data)
            .with_iat(now)
            .sign(username_password_hash);

        let query_request_dat = QueryRequest::as_dat(
            "UPDATE test_data_table SET im_data = ? WHERE id = ?;".to_string(),
            vec![
                QueryArg::new(query_arg::Value::String("updated_test_value1".to_string())),
                QueryArg::new(query_arg::Value::Int(1)),
            ],
        );

        let claims = Claims {
            iss: Iss::Client.into(),
            iat: now,
            exp: now + 30,
            sub: Sub::Mutate.into(),
            dat: Some(query_request_dat),
        };

        let result = get_proto_package_result(claims, &result_handler).await;
        let db_content = sqlx::query("SELECT * FROM test_data_table WHERE im_data = ?")
            .bind("updated_test_value1")
            .fetch_all(&db)
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_result_proto_package.unwrap());
        assert!(db_content.is_ok());
        assert_eq!(db_content.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_handle_mutate__mutate_data_insert_entry_success() {
        let db = setup_test_db().await;
        let username_password_hash = "test_hash";
        let result_handler = ProtoPackageResultHandler::new(2, username_password_hash, &db);
        let now = chrono::Utc::now().timestamp() as u64;

        let expected_result_proto_package = ProtoPackage::builder()
            .with_data(MutationResponse::as_dat(1, 2))
            .with_subject(Sub::Data)
            .with_iat(now)
            .sign(username_password_hash);

        let query_request_dat = QueryRequest::as_dat(
            "INSERT INTO test_data_table(im_data, im_data_too, im_data_aswell) VALUES(?, ?, ?);"
                .to_string(),
            vec![
                QueryArg::new(query_arg::Value::String("value1".to_string())),
                QueryArg::new(query_arg::Value::String("value2".to_string())),
                QueryArg::new(query_arg::Value::Int(123)),
            ],
        );

        let claims = Claims {
            iss: Iss::Client.into(),
            iat: now,
            exp: now + 30,
            sub: Sub::Mutate.into(),
            dat: Some(query_request_dat),
        };

        let result = get_proto_package_result(claims, &result_handler).await;
        let db_content = sqlx::query("SELECT * FROM test_data_table;")
            .fetch_all(&db)
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_result_proto_package.unwrap());
        assert!(db_content.is_ok());
        assert_eq!(db_content.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_handle_mutate__mutate_data_non_strict_table_insert_entry_success() {
        let db = setup_test_db().await;
        let username_password_hash = "test_hash";
        let result_handler = ProtoPackageResultHandler::new(2, username_password_hash, &db);
        let now = chrono::Utc::now().timestamp() as u64;

        let expected_result_proto_package = ProtoPackage::builder()
            .with_data(MutationResponse::as_dat(1, 2))
            .with_subject(Sub::Data)
            .with_iat(now)
            .sign(username_password_hash);

        let query_request_dat = QueryRequest::as_dat(
            "INSERT INTO test_data_table(im_data, im_data_too, im_data_aswell) VALUES(?, ?, ?);"
                .to_string(),
            vec![
                QueryArg::new(query_arg::Value::String("value1".to_string())),
                QueryArg::new(query_arg::Value::String("value2".to_string())),
                QueryArg::new(query_arg::Value::String("value3".to_string())),
            ],
        );

        let claims = Claims {
            iss: Iss::Client.into(),
            iat: now,
            exp: now + 30,
            sub: Sub::Mutate.into(),
            dat: Some(query_request_dat),
        };

        let result = get_proto_package_result(claims, &result_handler).await;
        let db_content = sqlx::query("SELECT * FROM test_data_table;")
            .fetch_all(&db)
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_result_proto_package.unwrap());
        assert!(db_content.is_ok());
        assert_eq!(db_content.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_handle_mutate__mutate_data_insert_entry_unknown_col_fail() {
        let db = setup_test_db().await;
        let username_password_hash = "test_hash";
        let result_handler = ProtoPackageResultHandler::new(2, username_password_hash, &db);

        let query_request_dat = QueryRequest::as_dat(
            "INSERT INTO test_data_table(im_data, im_data_too, im_data_yo) VALUES(?, ?, ?);"
                .to_string(),
            vec![
                QueryArg::new(query_arg::Value::String("value1".to_string())),
                QueryArg::new(query_arg::Value::String("value2".to_string())),
                QueryArg::new(query_arg::Value::String("value3".to_string())),
            ],
        );

        let claims = Claims {
            iss: Iss::Client.into(),
            iat: chrono::Utc::now().timestamp() as u64,
            exp: (chrono::Utc::now() + chrono::Duration::seconds(30)).timestamp() as u64,
            sub: Sub::Mutate.into(),
            dat: Some(query_request_dat),
        };

        let result = get_proto_package_result(claims, &result_handler).await;
        let db_content = sqlx::query("SELECT * FROM test_data_table;")
            .fetch_all(&db)
            .await;

        assert!(result.is_err());
        assert_eq!(
            result.expect_err("Should be DatabaseError"),
            DatabaseError::with_message("table test_data_table has no column named im_data_yo")
        );
        assert!(db_content.is_ok());
        assert_eq!(db_content.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_handle_mutate__mutate_data_strict_table_insert_entry_incorrect_type_fail() {
        let db = setup_test_db().await;
        let username_password_hash = "test_hash";
        let result_handler = ProtoPackageResultHandler::new(2, username_password_hash, &db);

        let query_request_dat = QueryRequest::as_dat(
            "INSERT INTO strict_test_data_table(im_data, im_data_too, im_data_aswell) VALUES(?, ?, ?);"
                .to_string(),
            vec![
                QueryArg::new(query_arg::Value::String("value1".to_string())),
                QueryArg::new(query_arg::Value::String("value2".to_string())),
                QueryArg::new(query_arg::Value::String("value3".to_string())),
            ],
        );

        let claims = Claims {
            iss: Iss::Client.into(),
            iat: chrono::Utc::now().timestamp() as u64,
            exp: (chrono::Utc::now() + chrono::Duration::seconds(30)).timestamp() as u64,
            sub: Sub::Mutate.into(),
            dat: Some(query_request_dat),
        };

        let result = get_proto_package_result(claims, &result_handler).await;
        let db_content = sqlx::query("SELECT * FROM strict_test_data_table;")
            .fetch_all(&db)
            .await;

        assert!(result.is_err());
        assert_eq!(
            result.expect_err("Should be DatabaseError"),
            DatabaseError::with_message(
                "cannot store TEXT value in INTEGER column strict_test_data_table.im_data_aswell"
            )
        );
        assert!(db_content.is_ok());
        assert_eq!(db_content.unwrap().len(), 0);
    }
    // MUTATE END
}
