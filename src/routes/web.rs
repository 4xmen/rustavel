
use axum::http::StatusCode;
use axum::response::IntoResponse;
use crate::core::routing::{Route,BuiltRoutes};
use crate::app::http::middleware::log_middleware;



pub fn web_routes() -> BuiltRoutes<AppState> {
    let mut route = Route::new();

    route.get("/",hello).name("welcome");
    route.post("/about",hello).name("about");
    // route.get("/about2",hello).name("about");
    route.any("/test",hello).name("test").middleware(log_middleware::log_request);

    route.group(|r| {
        r.name("api").prefix("/api").middleware(log_middleware::log_request);
        r.get("/users", hello).name("users");
    });

    route.build().unwrap_or_else(|e| {
        eprintln!("{:?}", e);
        std::process::exit(1);
    })
}

use axum::extract::State;
use crate::core::state::AppState; // import AppState

async fn hello(State(state): State<AppState>) -> impl IntoResponse {
    let welcome_url = state.routes.get("welcome").unwrap();
    println!("Welcome route: {}", welcome_url);
    let group_url = state.routes.get("api.users").unwrap();
    println!("Group route: {}", group_url);
    let x = vec!["hello", "world"];
    (StatusCode::OK, "hello world")
}