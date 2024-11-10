// use sqlx::{
//     database::Database,
//     query::Query,
//     sqlite::{ SqliteQueryResult, SqliteRow },
//     Encode,
//     Error,
//     Pool,
//     Sqlite,
//     Type,
// };

// // save for later
// // pub fn apply_query<'q, DB: Database, T>(
// //   query: Query<'q, DB, <DB as Database>::Arguments<'q>>,
// //   args: &'q [T]
// // )
// //   -> Query<'q, DB, <DB as Database>::Arguments<'q>>
// //   where DB: Database, T: Encode<'q, DB> + Type<DB> + 'q
// // {
// //   if args.len() == 0 {
// //       return query;
// //   }

// //   let [arg, tail @ ..] = args else {
// //       return query;
// //   };

// //   apply_query(query.bind(arg), tail)
// // }

// // pub fn apply_query<'q, DB: Database>(
// //   query: Query<'q, DB, <DB as Database>::Arguments<'q>>,
// //   args: &'q [QueryArg<'q>]
// // )
// //   -> Query<'q, DB, <DB as Database>::Arguments<'q>>
// //   where DB: Database, QueryArg<'q>: Encode<'q, DB> + Type<DB>
// // {
// //   if args.len() == 0 {
// //       return query;
// //   }

// //   let [arg, tail @ ..] = args else {
// //       return query;
// //   };

// //   apply_query(query.bind(arg), tail)
// // }

// // fn apply_query_bind<'q, DB: Database>(
// //     query: Query<'q, DB, <DB as Database>::Arguments<'q>>,
// //     args: &'q Vec<QueryArg<'q>>
// // )
// //     -> Query<'q, DB, <DB as Database>::Arguments<'q>>
// //     where DB: Database, QueryArg<'q>: Encode<'q, DB> + Type<DB>
// // {
// //     if args.len() == 0 {
// //         return query;
// //     }
// //     let (arg, tail) = args.split_first().unwrap();

// //     apply_query_bind(query.bind(arg), &tail.to_vec())
// // }

// // pub struct DatabaseConnection<'a, DB: Database> {
// //     pool: &'a Pool<Sqlite>,
// //     applied_query: Option<Query<'a, DB, <DB as Database>::Arguments<'a>>>,
// // }

// // impl<'a, DB: Database> DatabaseConnection<'a, DB> {
// //     pub fn new(pool: &'a Pool<Sqlite>) -> DatabaseConnection<'a, DB> {
// //         DatabaseConnection {
// //             pool: pool,
// //             applied_query: None,
// //         }
// //     }

// //     fn apply_query(self, query_str: &str, args: Vec<QueryArg>) -> Self where DB: Database, QueryArg<'a>: Encode<'a, DB> + Type<DB> {
// //         DatabaseConnection {
// //             applied_query: Some(apply_query_bind(sqlx::query(query_str), &args)),
// //             ..self
// //         }
// //     }
// // }

// // pub async fn execute_query<'a, DB: Database>(
// //     db_conn: DatabaseConnection<'a, DB>,
// //     query: &'a str,
// //     args: Vec<QueryArg<'a>>
// // ) -> Result<DB::QueryResult, Error> where QueryArg<'a>: Encode<'a, DB> + Type<DB> {
// //     db_conn.apply_query(query, args).applied_query.unwrap().execute(db_conn.pool).await
// // }

// pub fn apply_query<'q, DB: Database, T>(
//     query: Query<'q, DB, <DB as Database>::Arguments<'q>>,
//     args: &'q [T]
// )
//     -> Query<'q, DB, <DB as Database>::Arguments<'q>>
//     where DB: Database, T: Encode<'q, DB> + Type<DB> + 'q
// {
//     if args.len() == 0 {
//         return query;
//     }

//     let [arg, tail @ ..] = args else {
//         return query;
//     };

//     apply_query(query.bind(arg), tail)
// }

// pub fn apply_query2<'q, DB: Database>(
//     query: Query<'q, DB, <DB as Database>::Arguments<'q>>,
//     args: Option<Vec<QueryArg<'q>>>
// )
//     -> Query<'q, DB, <DB as Database>::Arguments<'q>>
//     where DB: Database, QueryArg<'q>: Encode<'q, DB> + Type<DB> + Clone
// {
//     if args.unwrap().len() == 0 {
//         return query;
//     }

//     let [arg, tail @ ..] = args.unwrap().as_slice() else {
//         return query;
//     };

//     match args.unwrap().split_first() {
//         Some((first, tail)) => {
//             return apply_query2(query.bind(*first), Some(tail.to_vec()));
//         }
//         None => {
//             return query;
//         }
//     };
//     // apply_query2(query.bind(12), Some(tail.to_vec()))
// }

// enum QueryArg<'a> {
//     Int(i32),
//     Float(f32),
//     Str(&'a str),
// }

// pub struct AppliedQuery<'a> {
//     query: &'a str,
//     args: Option<Vec<QueryArg<'a>>>,
// }

// impl<'a> AppliedQuery<'a> {
//     fn new(query: &'a str) -> AppliedQuery<'a> {
//         AppliedQuery {
//             query: query,
//             args: None,
//         }
//     }

//     fn with_args(self, args: Vec<QueryArg<'a>>) -> Self {
//         AppliedQuery {
//             args: Some(args),
//             ..self
//         }
//     }
// }

// pub fn test_applied(q: AppliedQuery) -> &str {
//     let unwrapped_args = q.args.unwrap();
//     let testing = match unwrapped_args.split_first() {
//         Some((first, tail)) => {
//             let int_val = if let QueryArg::Int(val) = *first { Some(val) } else { None };
//             let Float_val = if let QueryArg::Float(val) = *first { Some(val) } else { None };
//             let Str_val = if let QueryArg::Str(val) = *first { Some(val) } else { None };

//             (first, tail)
//         }
//         None => {
//             return q.query;
//         }
//     };

//     "Str::new()"
// }

// pub fn asdf() {
//     test_applied(
//         AppliedQuery::new("SELECT * FROM users5;").with_args(
//             vec![QueryArg::Int(1), QueryArg::Str("testing")]
//         )
//     );
// }

// pub async fn execute_query(
//     query: &str,
//     args: &[&str],
//     db: &Pool<Sqlite>
// ) -> Result<SqliteQueryResult, sqlx::Error> {
//     apply_query(sqlx::query(query), args).execute(db).await
// }

// pub async fn fetch_query(
//     query: &str,
//     args: &[&str],
//     db: &Pool<Sqlite>
// ) -> Result<Vec<SqliteRow>, sqlx::Error> {
//     apply_query(sqlx::query(query), args).fetch_all(db).await
// }

// pub async fn fetch_query2<'a, DB: Database, A>(
//     q: AppliedQuery<'a>,
//     db: &Pool<Sqlite>
// ) -> Result<Vec<SqliteRow>, sqlx::Error> where QueryArg<'a>: Encode<'a, DB> + Type<DB> + Clone {
//     apply_query2(sqlx::query(q.query), q.args).fetch_all(db).await
// }

use serde::{ Deserialize, Serialize };
use serde_json::{ json, Map as JsonMap, Value as JsonValue };
use sqlx::{ Database, Sqlite, SqlitePool, TypeInfo };
use sqlx::query::Query;
use sqlx::{ Column, Row };
use sqlx::sqlite::{ SqliteQueryResult, SqliteRow };

#[derive(Clone, Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum QueryArg<'a> {
    Int(i32),
    Float(f32),
    String(&'a str),
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
        AppliedQuery { args: Some(args), ..self }
    }
}

async fn fetch_query<'a>(
    q: AppliedQuery<'a>,
    db: &SqlitePool
) -> Result<Vec<SqliteRow>, sqlx::Error> {
    apply_query(sqlx::query(q.query), q.args).fetch_all(db).await
}

pub async fn execute_query<'a>(
    q: AppliedQuery<'a>,
    db: &SqlitePool
) -> Result<SqliteQueryResult, sqlx::Error> {
    apply_query(sqlx::query(q.query), q.args).execute(db).await
}

fn apply_query<'q>(
    query: Query<'q, Sqlite, <Sqlite as Database>::Arguments<'q>>,
    args: Option<Vec<QueryArg<'q>>>
) -> Query<'q, Sqlite, <Sqlite as Database>::Arguments<'q>> {
    let args = match args {
        Some(args) if !args.is_empty() => args,
        _ => {
            return query;
        }
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
    type_info: &'a str
) -> JsonValue {
    fn do_try_get<'b, T>(row: &'b SqliteRow, col_name: &'b str) -> JsonValue
        where
            JsonValue: std::convert::From<T>,
            T: sqlx::Type<sqlx::Sqlite> + sqlx::Decode<'b, sqlx::Sqlite>
    {
        row.try_get::<T, _>(col_name).map_or_else(|_| json!(null), JsonValue::from)
    }

    match type_info {
        "INTEGER" => do_try_get::<i32>(&row, col_name),
        "REAL" => do_try_get::<f32>(&row, col_name),
        "TEXT" => do_try_get::<String>(&row, col_name),
        _ => json!(null),
    }
}

pub async fn fetch_all_as_json<'a>(
    q: AppliedQuery<'a>,
    db: &SqlitePool
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
                    map_sqliterow_col_to_json_value(&row, column_name, TypeInfo::name(column.type_info()))
                );
            }

            JsonValue::Object(json_row)
        })
        .collect();

    Ok(result)
}
