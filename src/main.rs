use axum::{routing::get, Router};
use error::MyResult;
use serde_json::json;
mod error;
mod worlds;
use crate::worlds::routes::worlds_routes;

pub const UPLOADS_DIRECTORY: &str = "uploads";

#[tokio::main]
async fn main() {
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

async fn handle_dir() -> MyResult<axum::Json<serde_json::Value>>{
    let dir = get_current_working_dir().unwrap();
    let body = axum::Json(json!({
        "dir": dir
    }));

    Ok(body)
}

fn get_current_working_dir() -> std::io::Result<std::path::PathBuf> {
    std::env::current_dir()
}
