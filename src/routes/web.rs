
use axum::http::StatusCode;
use axum::response::IntoResponse;
use crate::core::routing::{Route,BuiltRoutes};
use crate::app::http::middleware::log_middleware;

pub fn web_routes() -> BuiltRoutes<AppState> {
    let mut route = Route::new();

    route.get("/",hello).name("welcome");
    route.post("/about",hello).name("about");
    route.get("/about",hello).name("about");
    route.any("/test",hello).name("test").middleware(log_middleware::log_request);

    route.build()
}

use axum::extract::State;
use crate::core::state::AppState; // import AppState

async fn hello(State(state): State<AppState>) -> impl IntoResponse {
    let welcome_url = state.routes.get("welcome").unwrap();
    println!("Welcome route: {}", welcome_url);
    let x = vec!["hello", "world"];
    (StatusCode::OK, "hello world")
}