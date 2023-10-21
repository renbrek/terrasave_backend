// use crate::error::{MyError, MyResult};
use axum::{
    body::{boxed, Body, BoxBody},
    http::{Request, Response, StatusCode},
    routing::get,
    Router,
};
use tower::ServiceExt;
use tower_http::services::ServeDir;

pub fn saves_routes() -> Router {
    Router::new().nest_service("/saves", ServeDir::new("assets"))
}

async fn api_saves_handler() -> Result<Response<BoxBody>, (StatusCode, String)> {
    let res = get_static_file().await?;
    println!("{:?}", res);

    if res.status() == StatusCode::NOT_FOUND {
        println!("NOT_FOUND")
    }
    Ok(res)
}

async fn get_static_file() -> Result<Response<BoxBody>, (StatusCode, String)> {
    let req = Request::builder().body(Body::empty()).unwrap();

    match ServeDir::new("assets").oneshot(req).await {
        Ok(res) => Ok(res.map(boxed)),
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", err),
        )),
    }
}
