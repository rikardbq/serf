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
