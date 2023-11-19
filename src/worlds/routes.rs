use crate::error::{MyResult, MyError};

use axum::{
    body::StreamBody,
    extract::{DefaultBodyLimit, Multipart, Path, State},
    http::{header, HeaderMap, StatusCode},
    response::{AppendHeaders, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};
use sqlx::{Pool, Row, Sqlite};
use tokio::{
    fs::{self},
    io::AsyncWriteExt,
};
use tokio_util::io::ReaderStream;
use tower_http::limit::RequestBodyLimitLayer;

use super::WorldFile;

const WORLD_FILE_DIR: &str = "terrasave_data/worlds";

pub fn worlds_routes(db: Pool<Sqlite>) -> Router {
    println!("{:?}", db);
    Router::new()
        .route("/worlds", get(handle_get_worlds))
        .route("/worlds/add_file", post(handle_add_world_file))
        .route("/worlds/get_file/:file_name", get(handle_get_world_file))
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024))
        .with_state(db)
}

async fn handle_add_world_file(
    State(db): State<Pool<Sqlite>>,
    header: HeaderMap,
    mut multipart: Multipart,
) -> MyResult<()> {
    let modified = header
        .get("x-last-modified")
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned()
        .parse::<i64>()
        .unwrap();

    let birthtime = header
        .get("x-birth-time")
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned()
        .parse::<i64>()
        .unwrap();

    while let Some(field) = multipart.next_field().await.unwrap() {
        let _ = fs::create_dir(WORLD_FILE_DIR).await;
        let file_name = match field.file_name() {
            Some(file_name) => file_name.to_string(), 
            None => "unknown".to_string(),
        };
        let path = format!("{}/{}", WORLD_FILE_DIR, file_name);
        let data = field.bytes().await.unwrap();
        let mut file = fs::File::create(&path).await.unwrap();
        let _ = file.write_all(&data).await.unwrap();

        let new_world_file: WorldFile =
            WorldFile::new(path.clone(), file_name.clone(), birthtime, modified);
        println!("new_world: {:?}", new_world_file);
        let result = sqlx::query(
            "INSERT INTO worlds (name, local_path, birthtime, modified) VALUES (?, ?, ?, ?)",
        )
        .bind(file_name)
        .bind(path)
        .bind(birthtime)
        .bind(modified)
        .execute(&db)
        .await
        .unwrap();

        println!("Query result: {:?}", result);

        let world_results = sqlx::query_as::<_, WorldFile>("SELECT * FROM worlds")
            .fetch_all(&db)
            .await
            .unwrap();

        for world in world_results {
            println!(
                "name: {}, local_path: {}, birthtime: {}, modified: {}",
                &world.name, &world.local_path, &world.birthtime, &world.modified
            );
        }
    }

    Ok(())
}

async fn handle_get_worlds(State(db): State<Pool<Sqlite>>) -> MyResult<Json<Value>> {
    let worlds_result = sqlx::query("SELECT * FROM worlds")
        .fetch_all(&db)
        .await
        .unwrap();

    for (idx, row) in worlds_result.iter().enumerate() {
        println!("[{}]: {:?}", idx, row.get::<String, &str>("name"));
    }

    let body = Json(json!({"ok": true, "world_files": "world_files"}));
    Ok(body)
}

async fn handle_get_world_file(Path(file_name): Path<String>) -> impl IntoResponse {
    println!("file_name:{}", &file_name);
    // `File` implements `AsyncRead`
    let file = match tokio::fs::File::open(format!("terrasave_data/worlds/{}", &file_name)).await {
        Ok(file) => file,
        Err(err) => return Err((StatusCode::NOT_FOUND, format!("File not found: {}", err))),
    };
    // convert the `AsyncRead` into a `Stream`
    let stream = ReaderStream::new(file);
    // convert the `Stream` into an `axum::body::HttpBody`
    let body = StreamBody::new(stream);

    let headers = AppendHeaders([
        (header::CONTENT_TYPE, "text/toml; charset=utf-8"),
        (header::CONTENT_DISPOSITION, "attachment"),
    ]);

    Ok((headers, body))
}
