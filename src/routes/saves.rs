// use crate::error::{MyError, MyResult};
use axum::{
    body::{boxed, Body, BoxBody},
    http::{Request, Response, StatusCode},
    routing::{get, get_service},
    Router, Json,
};
use serde_json::json;
use tower::ServiceExt;
use tower_http::services::ServeDir;

pub fn saves_routes() -> Router {
    Router::new()
        .nest_service("/worlds", get_service(ServeDir::new("assets/worlds")))
        .route("worlds/test", get(|| async { "test" }))
}
