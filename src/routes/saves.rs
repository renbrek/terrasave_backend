use axum::{Router, Json, routing::get};
use serde_json::{Value, json};

use crate::error::MyResult;

pub fn saves_routes() -> Router {
    Router::new().route("/saves", get(api_saves_handler))
}

async fn api_saves_handler() -> MyResult<Json<Value>>{
    // Create the success body.
	let body = Json(json!({
		"result": {
			"success": true
		}
	}));

	Ok(body)
}
