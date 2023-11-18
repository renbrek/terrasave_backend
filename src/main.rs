use axum::{routing::get, Router};
use error::MyResult;
use serde_json::json;
mod error;
mod worlds;
use crate::worlds::{routes::worlds_routes, WorldFile};
use sqlx::{migrate::MigrateDatabase, Row, Sqlite, SqlitePool};

pub const UPLOADS_DIRECTORY: &str = "uploads";

const DB_URL: &str = "sqlite://worlds.db";

#[tokio::main]
async fn main() {
    if !Sqlite::database_exists(DB_URL).await.unwrap_or(false) {
        println!("Creating database {}", DB_URL);
        match Sqlite::create_database(DB_URL).await {
            Ok(_) => println!("Create db success"),
            Err(error) => panic!("error: {}", error),
        }
    } else {
        println!("Database already exists");
    }

    let db = SqlitePool::connect(DB_URL).await.unwrap();
    // path, file_name, birthtime, modified
    let result = sqlx::query(
        "CREATE TABLE IF NOT EXISTS worlds (
            id INTEGER PRIMARY KEY NOT NULL,
            local_path VARCHAR(250) NOT NULL,
            name VARCHAR(250) NOT NULL,
            birthtime INTEGER NOT NULL,
            modified INTEGER NOT NULL
        );",
    )
    .execute(&db)
    .await
    .unwrap();
    println!("Create user table result: {:?}", result);

    
    let result = sqlx::query(
        "SELECT name
         FROM sqlite_schema
         WHERE type ='table';",
    )
    .fetch_all(&db)
    .await
    .unwrap();
    for (idx, row) in result.iter().enumerate() {
        println!("[{}]: {:?}", idx, row.get::<String, &str>("name"));
    }

    //dfasdasdf
    let result = sqlx::query("INSERT INTO worlds (name, local_path, birthtime, modified) VALUES (?, ?, ?, ?)")
        .bind("new_world")
        .bind("/assets/new_world.wld")
        .bind(213123123112_i64)
        .bind(131313131312_i64)
        .execute(&db)
        .await
        .unwrap();
    println!("Query result: {:?}", result);
    let world_results = sqlx::query_as::<_, WorldFile>("SELECT * FROM worlds")
        .fetch_all(&db)
        .await
        .unwrap();
    for world in world_results {
        println!("name: {}, local_path: {}, birthtime: {}, modified: {}", &world.name, &world.local_path, &world.birthtime, &world.modified);
    }
    //adfasdfasfas

    let app = Router::new()
        .merge(worlds_routes())
        .route("/dir", get(handle_dir))
        .route("/", get(|| async { "Hello, World!" }));

    println!("SERVER LISTENNING");
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handle_dir() -> MyResult<axum::Json<serde_json::Value>> {
    let dir = get_current_working_dir().unwrap();
    let body = axum::Json(json!({
        "dir": dir
    }));

    Ok(body)
}

fn get_current_working_dir() -> std::io::Result<std::path::PathBuf> {
    std::env::current_dir()
}
