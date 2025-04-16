#[allow(non_snake_case)]
pub mod db {
    use sqlx::{sqlite::SqliteArguments, Arguments, Execute};

    use crate::core::{
        db::{apply_query, AppliedQuery},
        serf_proto::{query_arg, QueryArg},
    };

    #[test]
    fn test_apply_query__has_bound_correct_query_argument_types() {
        let expected_query_string = "INSERT INTO test_table(some_string_val, some_integer_val, some_float_val, some_blob_val) VALUES(?, ?, ?, ?);";
        let expected_arg_string_val = "rikardbq";
        let expected_arg_int_val: i64 = 1234;
        let expected_arg_float_val: f64 = 1.234;
        let expected_arg_blob_val: Vec<u8> = vec![1, 2, 3];

        let mut sqlite_query_arguments = SqliteArguments::default();
        SqliteArguments::add(&mut sqlite_query_arguments, expected_arg_string_val).unwrap();
        SqliteArguments::add(&mut sqlite_query_arguments, expected_arg_int_val).unwrap();
        SqliteArguments::add(&mut sqlite_query_arguments, expected_arg_float_val).unwrap();
        SqliteArguments::add(&mut sqlite_query_arguments, expected_arg_blob_val).unwrap();

        let query_args_vec = vec![
            QueryArg::new(query_arg::Value::String("rikardbq".to_string())),
            QueryArg::new(query_arg::Value::Int(1234)),
            QueryArg::new(query_arg::Value::Float(1.234)),
            QueryArg::new(query_arg::Value::Blob(vec![1, 2, 3])),
        ];
        let applied_query = AppliedQuery::new("INSERT INTO test_table(some_string_val, some_integer_val, some_float_val, some_blob_val) VALUES(?, ?, ?, ?);").with_args(&query_args_vec);

        let sqlx_query = sqlx::query(applied_query.query);
        let mut query = apply_query(sqlx_query, applied_query.args);
        let query_args = query.take_arguments();

        assert!(query_args.is_ok());

        let query_args_option = query_args.unwrap();
        assert!(query_args_option.is_some());
        assert_eq!(query.sql(), expected_query_string);

        let query_arguments = format!("{:?}", query_args_option.unwrap());
        let expected_query_arguments = format!("{:?}", sqlite_query_arguments);

        assert_eq!(query_arguments, expected_query_arguments);
    }
}

#[allow(non_snake_case)]
pub mod state {
    use std::{any::Any, sync::Arc};

    use crate::core::{
        state::{AppState, User},
        util::create_db_connection,
    };

    #[test]
    fn test_app_state__get_user_exists() {
        let expected_user1 = User {
            username: "test_user".to_string(),
            username_hash: "test_user_hash".to_string(),
            username_password_hash: "some_other_hash".to_string(),
            db_access_rights: papaya::HashMap::new(),
        };
        let expected_user2 = User {
            username: "test_user2".to_string(),
            username_hash: "test_user2_hash".to_string(),
            username_password_hash: "some_other_hash".to_string(),
            db_access_rights: papaya::HashMap::new(),
        };

        let app_state = AppState {
            db_connections: Arc::new(papaya::HashMap::new()),
            users: Arc::new(papaya::HashMap::new()),
            db_max_connections: 32,
            db_max_idle_time: 3600,
            db_max_lifetime: 86400,
            db_path: String::from("testing_path"),
        };
        let users_guard = app_state.users_guard();
        let users = app_state.users.pin();
        users.insert(
            Arc::from("test_user_hash"),
            User {
                username: "test_user".to_string(),
                username_hash: "test_user_hash".to_string(),
                username_password_hash: "some_other_hash".to_string(),
                db_access_rights: papaya::HashMap::new(),
            },
        );
        users.insert(
            Arc::from("test_user2_hash"),
            User {
                username: "test_user2".to_string(),
                username_hash: "test_user2_hash".to_string(),
                username_password_hash: "some_other_hash".to_string(),
                db_access_rights: papaya::HashMap::new(),
            },
        );

        let user1_from_app_state = app_state.get_user("test_user_hash", &users_guard);
        let user2_from_app_state = app_state.get_user("test_user2_hash", &users_guard);

        assert!(user1_from_app_state.is_some());
        assert!(user2_from_app_state.is_some());
        assert_eq!(user1_from_app_state.unwrap(), &expected_user1);
        assert_eq!(user2_from_app_state.unwrap(), &expected_user2)
    }

    #[test]
    fn test_app_state__get_user_not_exists() {
        let app_state = AppState {
            db_connections: Arc::new(papaya::HashMap::new()),
            users: Arc::new(papaya::HashMap::new()),
            db_max_connections: 32,
            db_max_idle_time: 3600,
            db_max_lifetime: 86400,
            db_path: String::from("testing_path"),
        };
        let users_guard = app_state.users_guard();

        let user1_from_app_state = app_state.get_user("test_user_hash", &users_guard);
        let user2_from_app_state = app_state.get_user("test_user2_hash", &users_guard);

        assert!(user1_from_app_state.is_none());
        assert!(user2_from_app_state.is_none());
    }

    #[test]
    fn test_app_state__get_user_db_access_rights_exists() {
        let expected_user1_access_right = 3u8;
        let expected_user2_access_right = 2u8;

        let user1 = User {
            username: "test_user".to_string(),
            username_hash: "test_user_hash".to_string(),
            username_password_hash: "some_other_hash".to_string(),
            db_access_rights: papaya::HashMap::new(),
        };
        let user1_access_rights = &user1.db_access_rights.pin();
        user1_access_rights.insert(Arc::from("test_db_name"), 3);

        let user2 = User {
            username: "test_user2".to_string(),
            username_hash: "test_user2_hash".to_string(),
            username_password_hash: "some_other_hash".to_string(),
            db_access_rights: papaya::HashMap::new(),
        };
        let user2_access_rights = &user2.db_access_rights.pin();
        user2_access_rights.insert(Arc::from("test_db_name"), 2);

        let user1_access_right = user1.get_access_right("test_db_name");
        let user2_access_right = user2.get_access_right("test_db_name");

        assert_eq!(user1_access_right, expected_user1_access_right);
        assert_eq!(user2_access_right, expected_user2_access_right);
    }

    #[test]
    fn test_app_state__get_user_db_access_rights_not_exists() {
        let expected_user_access_right = 0u8;

        let user = User {
            username: "test_user".to_string(),
            username_hash: "test_user_hash".to_string(),
            username_password_hash: "some_other_hash".to_string(),
            db_access_rights: papaya::HashMap::new(),
        };

        let user_access_right = user.get_access_right("test_db_name");

        assert_eq!(user_access_right, expected_user_access_right);
    }

    #[tokio::test]
    async fn test_app_state__create_and_insert_db_connection_success() {
        let app_state = AppState {
            db_connections: Arc::new(papaya::HashMap::new()),
            users: Arc::new(papaya::HashMap::new()),
            db_max_connections: 32,
            db_max_idle_time: 3600,
            db_max_lifetime: 86400,
            db_path: String::from("testing_path"),
        };
        let db_connections_guard = app_state.db_connections_guard();
        let created_db_connection = create_db_connection(
            "sqlite::memory:",
            app_state.db_max_connections,
            app_state.db_max_idle_time,
            app_state.db_max_lifetime,
        )
        .await;
        assert!(created_db_connection.is_ok());

        app_state.insert_db_connection(
            "test_db_name",
            created_db_connection.unwrap(),
            &db_connections_guard,
        );
        assert!(true);
    }

    #[tokio::test]
    async fn test_app_state__create_and_get_db_connection_success() {
        let app_state = AppState {
            db_connections: Arc::new(papaya::HashMap::new()),
            users: Arc::new(papaya::HashMap::new()),
            db_max_connections: 32,
            db_max_idle_time: 3600,
            db_max_lifetime: 86400,
            db_path: String::from("testing_path"),
        };
        let db_connections_guard = app_state.db_connections_guard();
        let created_db_connection = create_db_connection(
            "sqlite::memory:",
            app_state.db_max_connections,
            app_state.db_max_idle_time,
            app_state.db_max_lifetime,
        )
        .await;
        assert!(created_db_connection.is_ok());

        let unwrapped_created_db_connection = created_db_connection.unwrap();
        app_state.insert_db_connection(
            "test_db_name",
            unwrapped_created_db_connection.clone(),
            &db_connections_guard,
        );
        assert!(true);

        let db_connection = app_state.get_db_connection("test_db_name", &db_connections_guard);
        assert!(db_connection.is_some());
        let unwrapped_db_connection = db_connection.unwrap();
        assert_eq!(
            unwrapped_db_connection.type_id(),
            unwrapped_created_db_connection.type_id()
        );
    }
}
