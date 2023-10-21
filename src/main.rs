use axum::{
    routing::get,
    Router,
};

mod routes;
mod error;
use routes::saves::saves_routes;

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new().merge(saves_routes()).route("/", get(|| async { "Hello, World!" }));

    println!("SERVER LISTENNING");
    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
