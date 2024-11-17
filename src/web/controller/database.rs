use std::sync::Arc;
use std::time::Duration;

use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

use crate::{
    core::{
        db::{execute_query, fetch_all_as_json, AppliedQuery},
        state::{AppState, Usr},
    },
    web::{
        jwt::{decode_token, generate_claims, generate_token, Iss, RequestQuery, Sub},
        request::{RequestBody, ResponseResult},
    },
};
/*
save for later testing


#[post("/{database}")]
async fn echo(
    data: web::Data<AppState<SqlitePool>>,
    path: web::Path<String>,
    req_body: String,
) -> impl Responder {
    let database = path.into_inner();
    let database_connections_clone: Arc<Mutex<HashMap<String, SqlitePool>>> =
        Arc::clone(&data.database_connections);
    let mut database_connections = database_connections_clone
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    println!(
        "hello {}, {}",
        database,
        database_connections.keys().count()
    );
    if !database_connections.contains_key(&database) {
        println!(
            "database connection is not opened, trying to open database {}",
            database
        );
        if let Ok(pool) =
            SqlitePool::connect(format!("sqlite:{}.db?mode=json", database).as_str()).await
        {
            database_connections.insert(database.clone(), pool);
        } else {
            println!();
        }
    }
    let db = &database_connections[&database];
    let result = execute_query(
        AppliedQuery::new(
            "CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY NOT NULL, name VARCHAR(250) NOT NULL);"
        ),
        &db
    ).await.unwrap();
    println!("test {:?}", result);

    let result2 = sqlx
        ::query(
            "CREATE TABLE IF NOT EXISTS users2 (id INTEGER PRIMARY KEY NOT NULL, name VARCHAR(250), namu VARCHAR(250) NOT NULL);"
        )
        .execute(db).await
        .unwrap();
    println!("test {:?}", result2);

    let result3 = execute_query(
        AppliedQuery::new(
            "CREATE TABLE IF NOT EXISTS users5 (id INTEGER PRIMARY KEY NOT NULL, name VARCHAR(250), namu VARCHAR(250), asdf INTEGER);"
        ),
        &db
    ).await.unwrap();

    println!("test {:?}", result3);

    execute_query(
        AppliedQuery::new("INSERT INTO users5 (name, namu, asdf) VALUES (?, ?, ?)").with_args(
            vec![
                QueryArg::String("hello"),
                QueryArg::String("World"),
                QueryArg::Int(32),
            ],
        ),
        &db,
    )
    .await
    .unwrap();

    // let result4 = fetch_query(AppliedQuery::new("SELECT * FROM users5;"), &db).await.unwrap();
    // for (idx, row) in result4.iter().enumerate() {
    //     println!(
    //         "[{}]: {:?} {:?} {:?}",
    //         idx,
    //         row.get::<String, &str>("name"),
    //         row.get::<String, &str>("namu"),
    //         row.get::<i32, &str>("asdf")
    //     );
    // }

    // get the usr data
    let usr_clone: Arc<Mutex<HashMap<String, Usr>>> = Arc::clone(&data.usr);
    let usr = usr_clone.lock().unwrap_or_else(PoisonError::into_inner);

    let user_entry_for_id =
        &usr["b1a74559bea16b1521205f95f07a25ea2f09f49eb4e265fa6057036d1dff7c22"];
    println!("testing here usr = {:?}", user_entry_for_id);
    HttpResponse::Ok().body(req_body)
}


*/

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(handle_database_post);
}

#[post("/{database}")]
async fn handle_database_post(
    req: HttpRequest,
    data: web::Data<AppState>,
    path: web::Path<String>,
    req_body: web::Json<RequestBody>,
) -> impl Responder {
    let header_u_ = match req.headers().get("u_") {
        Some(hdr) => hdr.to_str().unwrap(),
        _ => {
            return HttpResponse::BadRequest()
                .json(ResponseResult::new().error("ERROR=MissingHeader/s[ u_ ]"));
        }
    };

    let usr_clone: Arc<papaya::HashMap<String, Usr>> = Arc::clone(&data.usr);
    let usr = usr_clone.pin();
    let user_entry_for_id = match usr.get(header_u_) {
        Some(u) => u,
        None => {
            return HttpResponse::Unauthorized()
                .json(ResponseResult::new().error("ERROR=UnknownUser"))
        }
    };
    let database = path.into_inner();
    let database_connections_clone: Arc<papaya::HashMap<String, SqlitePool>> =
        Arc::clone(&data.database_connections);
    let database_connections = database_connections_clone.pin();

    if !database_connections.contains_key(&database) {
        println!(
            "database connection is not opened, trying to open database {}",
            database
        );
        if let Ok(pool) = SqlitePoolOptions::new()
            .max_connections(data.db_max_conn)
            .idle_timeout(Duration::from_secs(data.db_max_idle_time))
            .connect(&format!("sqlite:{}.db", database))
            .await
        {
            database_connections.insert(database.clone(), pool);
        } else {
            return HttpResponse::NotFound().json(
                ResponseResult::new().error(&format!("ERROR=Database not found for {}", database)),
            );
            // panic!("ERROR=No database found for {}", database);
        }
    }

    let db = database_connections.get(&database).unwrap();
    let token = &req_body.payload;
    let decoded_token = match decode_token(&token, &user_entry_for_id.up_hash) {
        Ok(dec) => dec,
        Err(err) => {
            return HttpResponse::NotAcceptable()
                .json(ResponseResult::new().error(&format!("ERROR={:?}", err.kind())));
        }
    };
    let claims = decoded_token.claims;

    if claims.iss != Iss::C_ {
        return HttpResponse::NotAcceptable()
            .json(ResponseResult::new().error("ERROR=InvalidIssuer"));
    }

    let dat: RequestQuery = serde_json::from_str(&claims.dat).unwrap();
    let res = match claims.sub {
        Sub::M_ => {
            let result = execute_query(AppliedQuery::new(&dat.base_query).with_args(dat.parts), db)
                .await
                .unwrap()
                .rows_affected();
            let claims = generate_claims(serde_json::to_string(&result).unwrap(), Sub::D_);
            let token = generate_token(claims, &user_entry_for_id.up_hash).unwrap();

            println!("{}", serde_json::to_string_pretty(&result).unwrap());
            HttpResponse::Ok().json(ResponseResult::new().payload(token))
        }
        Sub::F_ => {
            let result =
                fetch_all_as_json(AppliedQuery::new(&dat.base_query).with_args(dat.parts), db)
                    .await
                    .unwrap();
            let claims = generate_claims(serde_json::to_string(&result).unwrap(), Sub::D_);
            let token = generate_token(claims, &user_entry_for_id.up_hash).unwrap();

            println!("{}", serde_json::to_string_pretty(&result).unwrap());
            HttpResponse::Ok().json(ResponseResult::new().payload(token))
        }
        _ => {
            HttpResponse::NotAcceptable().json(ResponseResult::new().error("ERROR=InvalidSubject"))
        }
    };

    res
}
