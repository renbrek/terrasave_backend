use axum::{http::StatusCode, response::IntoResponse};

pub type MyResult<T> = core::result::Result<T, MyError>;

#[derive(Debug)]
pub enum MyError {
    InternalError,
}

impl IntoResponse for MyError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_CLIENT_ERROR").into_response()
    }
}

impl std::fmt::Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for MyError {}
