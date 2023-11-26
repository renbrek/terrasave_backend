use std::io;

use crate::error::{MyResult, MyError};

use axum::{
    body::{StreamBody, Bytes},
    extract::{DefaultBodyLimit, Multipart, Path, State, BodyStream},
    http::{header, HeaderMap, StatusCode},
    response::{AppendHeaders, IntoResponse},
    routing::{get, post},
    Json, Router, BoxError,
};
use futures::{Stream, TryStreamExt};
use serde_json::{json, Value};
use sqlx::{Pool, Sqlite};
use tokio::{
    fs::{self, File},
    io::{AsyncWriteExt, BufWriter},
};
use tokio_util::io::{ReaderStream, StreamReader};

use super::WorldFile;

const WORLD_FILE_DIR: &str = "data/worlds";

pub fn worlds_routes(db: Pool<Sqlite>) -> Router {
    println!("{:?}", db);
    Router::new()
        .route("/worlds", get(handle_get_worlds))
        .route("/worlds/add_file", post(handle_add_world_file))
        .route("/worlds/get_file/:file_name", get(handle_get_world_file))
        .route("/worlds/upload/:file_name", post(handle_upload))
        .with_state(db)
}

async fn handle_upload(
    body: BodyStream,
) -> Result<(), (StatusCode, String)> {
    println!("upload");
    stream_to_file(body).await
}

// Save a `Stream` to a file
async fn stream_to_file<S, E>(stream: S) -> Result<(), (StatusCode, String)>
where
    S: Stream<Item = Result<Bytes, E>>,
    E: Into<BoxError>,
{
    let path = "name";
    // if !path_is_valid(path) {
    //     return Err((StatusCode::BAD_REQUEST, "Invalid path".to_owned()));
    // }
    let _ = fs::create_dir(WORLD_FILE_DIR).await;

    async {
        // Convert the stream into an `AsyncRead`.
        let body_with_io_error = stream.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
        let body_reader = StreamReader::new(body_with_io_error);
        futures::pin_mut!(body_reader);

        // Create the file. `File` implements `AsyncWrite`.
        
        let path = std::path::Path::new(WORLD_FILE_DIR).join(path);
        let mut file = BufWriter::new(File::create(&path).await?);

        // Copy the body into the file.
        tokio::io::copy(&mut body_reader, &mut file).await?;
        let meta = std::fs::metadata(&path).unwrap();
        let modif = meta.modified().unwrap();
        println!("new f modif: {:?}", modif);

        Ok::<_, io::Error>(())
    }
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))

    // let created_world = WorldFile::new(path, name, birthtime, modified)
}

async fn handle_add_world_file( State(db): State<Pool<Sqlite>>,
    header: HeaderMap,
    mut multipart: Multipart,
) -> MyResult<Json<Value>> {
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
            None => {
                println!("error");
                return Err(MyError::InternalError)
            },
        };

        let check_file_name = sqlx::query_as::<_, WorldFile>("SELECT * FROM worlds WHERE name=(?)").bind(&file_name).fetch_all(&db).await.unwrap();

        println!("{:?}, len: {}", check_file_name, check_file_name.len());

        if check_file_name.len() > 0 {
            println!("SHOULD RETURN OK RESPONSE");
            return Ok(Json(json!({ "ok": false })));
        }

        let path = format!("{}/{}", WORLD_FILE_DIR, file_name);
        let data = field.bytes().await.unwrap();
        let mut file = fs::File::create(&path).await.unwrap();
        let _ = file.write_all(&data).await.unwrap();
        let new_world_file: WorldFile =
            WorldFile::new(path.clone(), file_name.clone(), birthtime, modified);
        // Обрабатывать случай, когда файл уже существует
        let _result = sqlx::query(
            "INSERT INTO worlds (name, local_path, birthtime, modified) VALUES (?, ?, ?, ?)",
        )
        .bind(new_world_file.name)
        .bind(new_world_file.local_path)
        .bind(new_world_file.birthtime)
        .bind(new_world_file.modified)
        .execute(&db)
        .await
        .unwrap();

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
    Ok(Json(json!({ "ok": true })))
}

async fn handle_get_worlds(State(db): State<Pool<Sqlite>>) -> MyResult<Json<Value>> {
    let worlds_result = sqlx::query_as::<_,WorldFile>("SELECT * FROM worlds")
        .fetch_all(&db)
        .await
        .unwrap();
    let body = Json(json!({"ok": true, "worlds": worlds_result}));
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
