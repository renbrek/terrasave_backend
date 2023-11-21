use axum::{routing::get, Router};
use tokio::fs;
mod error;
mod worlds;
use crate::worlds::routes::worlds_routes;
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};

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
    let _create_worlds_table = sqlx::query(
        "CREATE TABLE IF NOT EXISTS worlds (
            name VARCHAR(250) PRIMARY KEY NOT NULL,
            local_path VARCHAR(250) NOT NULL,
            birthtime INTEGER NOT NULL,
            modified INTEGER NOT NULL
        );",
    )
    .execute(&db)
    .await
    .unwrap();

    let _ = fs::create_dir("data").await;

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .merge(worlds_routes(db.clone()));

    println!("SERVER LISTENNING");
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
