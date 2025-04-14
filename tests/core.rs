// INTEGRATION TESTS

#[allow(non_snake_case)]
#[cfg(test)]
pub mod db {
    use serf::core::db::{fetch_all_as_json, AppliedQuery};
    
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
                im_text_data	TEXT,
                im_float_data   REAL,
                im_blob_data    BLOB,
                im_null_data    TEXT
            ) STRICT;
        "#,
        )
        .execute(&db)
        .await
        .expect("Failed to create test data table");

        sqlx::query(
            r#"
            INSERT INTO test_data_table(im_text_data, im_float_data, im_blob_data) VALUES(?, ?, ?);
            "#,
        )
        .bind("text_value")
        .bind(1.234)
        .bind(vec![1, 2, 3])
        .execute(&db)
        .await
        .expect("Failed to add test entry to test data table");

        db
    }

    #[tokio::test]
    async fn test_map_sqliterow_col_to_json_value__correctly_map_types() {
        let db = setup_test_db().await;

        let applied_query = AppliedQuery::new("SELECT NULLIF(im_null_data, '') im_null_data, id, im_text_data, im_float_data, im_blob_data FROM test_data_table;");
        let data_res = fetch_all_as_json(applied_query, &db).await;
        assert!(data_res.is_ok());

        let data = data_res.unwrap();
        assert!(data.is_array());

        let data_arr = data.as_array();
        assert!(data_arr.is_some());

        data_arr.unwrap().iter().for_each(|x| {
            let int_data = x.get("id").unwrap();
            let text_data = x.get("im_text_data").unwrap();
            let float_data = x.get("im_float_data").unwrap();
            let blob_data = x.get("im_blob_data").unwrap();
            let null_data = x.get("im_null_data").unwrap();

            assert!(int_data.is_number());
            assert!(text_data.is_string());
            assert!(float_data.is_f64());
            assert!(blob_data.is_array());
            assert!(null_data.is_null());
        });
    }
}
