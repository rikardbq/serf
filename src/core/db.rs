use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::{json, Map as JsonMap, Value as JsonValue};
use sqlx::query::Query;
use sqlx::sqlite::{SqliteQueryResult, SqliteRow};
use sqlx::{Column, Executor, Row};
use sqlx::{Database, Sqlite, SqlitePool, TypeInfo};

#[derive(Clone, Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum QueryArg<'a> {
    Int(i32),
    Float(f32),
    #[serde(borrow)]
    String(&'a str),
}

impl<'a> fmt::Display for QueryArg<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct AppliedQuery<'a> {
    query: &'a str,
    args: Option<Vec<QueryArg<'a>>>,
}

impl<'a> AppliedQuery<'a> {
    pub fn new(query: &'a str) -> AppliedQuery<'a> {
        AppliedQuery { query, args: None }
    }

    pub fn with_args(self, args: Vec<QueryArg<'a>>) -> Self {
        AppliedQuery {
            args: Some(args),
            ..self
        }
    }
}

async fn fetch_query<'a>(
    q: AppliedQuery<'a>,
    db: &SqlitePool,
) -> Result<Vec<SqliteRow>, sqlx::Error> {
    apply_query(sqlx::query(q.query), q.args)
        .fetch_all(db)
        .await
}

pub async fn execute_query<'a, T>(
    q: AppliedQuery<'a>,
    db: T,
) -> Result<SqliteQueryResult, sqlx::Error>
where
    T: Executor<'a, Database = Sqlite>,
{
    apply_query(sqlx::query(q.query), q.args).execute(db).await
}

fn apply_query<'q>(
    query: Query<'q, Sqlite, <Sqlite as Database>::Arguments<'q>>,
    args: Option<Vec<QueryArg<'q>>>,
) -> Query<'q, Sqlite, <Sqlite as Database>::Arguments<'q>> {
    let args = match args {
        Some(args) if !args.is_empty() => args,
        _ => return query,
    };

    let [first, tail @ ..] = args.as_slice() else {
        return query;
    };

    let query = match first {
        QueryArg::Int(val) => query.bind(*val),
        QueryArg::Float(val) => query.bind(*val),
        QueryArg::String(val) => query.bind(*val),
    };

    apply_query(query, Some(tail.to_vec()))
}

// may need to do something similar later where i map struct as enum values with pre-processing of values
// impl<'a> QueryArg<'a> {
//     fn as_value(&self) -> &(dyn sqlx::Encode<'a, Sqlite> + 'a) {
//         match self {
//             QueryArg::Int(val) => val,
//             QueryArg::Float(val) => val,
//             QueryArg::Str(val) => val,
//         }
//     }
// }

fn map_sqliterow_col_to_json_value<'a>(
    row: &'a SqliteRow,
    col_name: &'a str,
    type_info: &'a str,
) -> JsonValue {
    fn do_try_get<'b, T>(row: &'b SqliteRow, col_name: &'b str) -> JsonValue
    where
        JsonValue: std::convert::From<T>,
        T: sqlx::Type<sqlx::Sqlite> + sqlx::Decode<'b, sqlx::Sqlite>,
    {
        row.try_get::<T, _>(col_name)
            .map_or_else(|_| json!(null), JsonValue::from)
    }

    match type_info {
        "INTEGER" => do_try_get::<i32>(row, col_name),
        "REAL" => do_try_get::<f32>(row, col_name),
        "TEXT" => do_try_get::<String>(row, col_name),
        "NULL" => do_try_get::<JsonValue>(row, col_name),
        _ => json!(null),
    }
}

pub async fn fetch_all_as_json<'a>(
    q: AppliedQuery<'a>,
    db: &SqlitePool,
) -> Result<Vec<JsonValue>, sqlx::error::Error> {
    let rows = fetch_query(q, db).await?;
    let result = rows
        .into_iter()
        .map(|row| {
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

            JsonValue::Object(json_row)
        })
        .collect();

    Ok(result)
}
