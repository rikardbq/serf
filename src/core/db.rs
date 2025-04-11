use serde_json::{json, Map as JsonMap, Value as JsonValue};
use sqlx::query::Query;
use sqlx::sqlite::{SqliteQueryResult, SqliteRow};
use sqlx::{Column, Executor, Row};
use sqlx::{Database, Sqlite, SqlitePool, TypeInfo};

use crate::core::serf_proto::{query_arg, QueryArg};

pub struct AppliedQuery<'a> {
    pub query: &'a str,
    pub args: Option<&'a [QueryArg]>,
}

impl<'a> AppliedQuery<'a> {
    pub fn new(query: &'a str) -> AppliedQuery<'a> {
        AppliedQuery { query, args: None }
    }

    pub fn with_args(self, args: &'a [QueryArg]) -> Self {
        AppliedQuery {
            args: Some(args),
            ..self
        }
    }
}

async fn fetch_query<'a>(
    q: AppliedQuery<'a>,
    db: &'a SqlitePool,
) -> Result<Vec<SqliteRow>, sqlx::error::Error> {
    apply_query(sqlx::query(q.query), q.args)
        .fetch_all(db)
        .await
}

pub async fn execute_query<'a, T>(
    q: AppliedQuery<'a>,
    db: T,
) -> Result<SqliteQueryResult, sqlx::error::Error>
where
    T: Executor<'a, Database = Sqlite>,
{
    apply_query(sqlx::query(q.query), q.args).execute(db).await
}

pub fn apply_query<'q>(
    query: Query<'q, Sqlite, <Sqlite as Database>::Arguments<'q>>,
    args: Option<&'q [QueryArg]>,
) -> Query<'q, Sqlite, <Sqlite as Database>::Arguments<'q>> {
    let args = match args {
        Some(args) if !args.is_empty() => args,
        _ => return query,
    };

    let mut query = query;
    for x in args {
        if let Some(query_arg_val) = &x.value {
            query = match query_arg_val {
                query_arg::Value::Int(val) => query.bind(val),
                query_arg::Value::Float(val) => query.bind(val),
                query_arg::Value::String(val) => query.bind(val),
                query_arg::Value::Blob(val) => query.bind(val),
            };
        }
    }

    query
}

fn map_row_col_type<'b, T>(row: &'b SqliteRow, col_name: &'b str) -> JsonValue
where
    JsonValue: std::convert::From<T>,
    T: sqlx::Type<sqlx::Sqlite> + sqlx::Decode<'b, sqlx::Sqlite>,
{
    row.try_get::<T, _>(col_name)
        .map_or_else(|_| json!(null), JsonValue::from)
}

pub fn map_sqliterow_col_to_json_value<'a>(
    row: &'a SqliteRow,
    col_name: &'a str,
    type_info: &'a str,
) -> JsonValue {
    match type_info {
        "BLOB" => map_row_col_type::<Vec<u8>>(row, col_name),
        "INTEGER" => map_row_col_type::<i64>(row, col_name),
        "REAL" => map_row_col_type::<f64>(row, col_name),
        "TEXT" => map_row_col_type::<String>(row, col_name),
        "NULL" => map_row_col_type::<JsonValue>(row, col_name),
        _ => json!(null),
    }
}

pub async fn fetch_all_as_json<'a>(
    q: AppliedQuery<'a>,
    db: &'a SqlitePool,
) -> Result<JsonValue, sqlx::error::Error> {
    let rows = fetch_query(q, db).await?;
    let mut json_array = JsonValue::Array(vec![]);

    if let JsonValue::Array(ref mut arr) = json_array {
        rows.into_iter().for_each(|row| {
            let mut json_row = JsonMap::new();

            for column in row.columns() {
                let column_name = column.name();

                json_row.insert(
                    column_name.to_string(),
                    map_sqliterow_col_to_json_value(
                        &row,
                        column_name,
                        TypeInfo::name(column.type_info()),
                    ),
                );
            }

            arr.push(JsonValue::Object(json_row));
        });
    }

    Ok(json_array)
}
