use std::{
    io::{self, Error},
    path::PathBuf,
};

use crate::UPLOADS_DIRECTORY;
use axum::{
    body::{Bytes, StreamBody},
    extract::{BodyStream, DefaultBodyLimit, Multipart, Path},
    http::{header, HeaderMap, StatusCode},
    response::{AppendHeaders, IntoResponse},
    routing::{get, post},
    BoxError, Router,
};
use futures::{Stream, TryStreamExt};
use tokio::{
    fs::{self, File},
    io::{AsyncWriteExt, BufWriter},
};
use tokio_util::io::{ReaderStream, StreamReader};
use tower_http::limit::RequestBodyLimitLayer;

use super::WorldFile;

const WORLD_FILE_DIR: &str = "assets/worlds";
pub fn worlds_routes() -> Router {
    Router::new()
        // .route("/worlds", get(handle_get_worlds))
        .route("/worlds/add_file", post(handle_add_world_file))
        .route("/worlds/get_file/:file_name", get(handle_get_world_file))
        .route("/worlds/upload/:file_name", post(handle_upload))
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024))
}

async fn handle_upload(
    Path(file_name): Path<String>,
    body: BodyStream,
) -> Result<(), (StatusCode, String)> {
    println!("upload");
    stream_to_file(&file_name, body).await
}

// Save a `Stream` to a file
async fn stream_to_file<S, E>(path: &str, stream: S) -> Result<(), (StatusCode, String)>
where
    S: Stream<Item = Result<Bytes, E>>,
    E: Into<BoxError>,
{
    // if !path_is_valid(path) {
    //     return Err((StatusCode::BAD_REQUEST, "Invalid path".to_owned()));
    // }

    async {
        // Convert the stream into an `AsyncRead`.
        let body_with_io_error = stream.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
        let body_reader = StreamReader::new(body_with_io_error);
        futures::pin_mut!(body_reader);

        // Create the file. `File` implements `AsyncWrite`.
        let path = std::path::Path::new(UPLOADS_DIRECTORY).join(path);
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

async fn handle_add_world_file(header: HeaderMap, mut multipart: Multipart) {
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
        let file_name = field.file_name().unwrap().to_string();
        let path = format!("{}/{}", WORLD_FILE_DIR, file_name);
        let data = field.bytes().await.unwrap();
        let mut file = fs::File::create(&path).await.unwrap();
        let _ = file.write_all(&data).await.unwrap();

        let new_world_file: WorldFile = WorldFile::new(path, file_name, birthtime, modified);
        println!("new_world: {:?}", new_world_file);
    }
}
// async fn handle_get_worlds() -> MyResult<Json<Value>> {
//     let worlds = match get_worlds_paths().await {
//         Ok(worlds) => worlds,
//         Err(_) => return Err(MyError::InternalError),
//     };
//
//     let world_files: Vec<WorldFile> = map_path_bufs_to_world_files(worlds);
//     let body = Json(json!({"ok": true, "world_files": world_files}));
//     for world in &world_files {
//         let meta = fs::metadata(&world.local_path).await.unwrap();
//         let modified = meta.modified().unwrap();
//         println!("{}- {:#?}", &world.local_path, modified);
//     }
//     Ok(body)
// }

async fn get_worlds_paths() -> Result<Vec<PathBuf>, Error> {
    let mut worlds: Vec<PathBuf> = vec![];
    let mut worlds_dir = fs::read_dir("assets/worlds").await?;
    while let Some(world) = worlds_dir.next_entry().await? {
        worlds.push(world.path());
    }
    Ok(worlds)
}

async fn handle_get_world_file(Path(file_name): Path<String>) -> impl IntoResponse {
    println!("file_name:{}", &file_name);
    // `File` implements `AsyncRead`
    let file = match tokio::fs::File::open(format!("assets/worlds/{}", &file_name)).await {
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

// fn map_path_bufs_to_world_files(files: Vec<PathBuf>) -> Vec<WorldFile> {
//     files
//         .into_iter()
//         .map(|item| {
//             let item = item.to_str().unwrap().to_owned();
//             let meta = std::fs::metadata(&item).unwrap();
//             let birthtime = meta.created().unwrap();
//
//             // let SystemTime(a) = birthtime;
//             let modified = meta.modified().unwrap();
//             WorldFile::new(item.to_owned(), item.to_owned(), birthtime,  modified)
//         })
//         .collect()
// }
