use axum::{Router, routing::get, Json};
use serde_json::{json, Value};

use crate::error::MyResult;

pub fn worlds_routes() -> Router {
    Router::new().route("/worlds", get(get_worlds_handler))
}

async fn get_worlds_handler() -> MyResult<Json<Value>>{
    let body = Json(json!({
        "worlds": []
    }));

    Ok(body)
}