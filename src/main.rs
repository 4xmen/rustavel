mod routes;
mod core;

use std::sync::Arc;
use axum::Router;

use crate::core::state::AppState;

#[tokio::main]
async fn main() {
    let built = routes::web::web_routes();

    println!("Starting server on http://localhost:3000");

    let routes_map = Arc::new(built.names.clone());

    println!("route list: {:?}", built.names);

    let state = AppState { routes: routes_map };

    // type annotation رو بردار
    let app = built.router.with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}