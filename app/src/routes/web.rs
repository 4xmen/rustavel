use crate::http::controllers::todo_controller;
use crate::http::middleware::log_middleware;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use rustavel_core::routing::{BuiltRoutes, Route};

pub fn web_routes() -> BuiltRoutes<AppState> {
    let mut route = Route::new();

    route.get("/", hello).name("welcome");
    route.get("/about", hello).name("about");
    // route.get("/about2",hello).name("about");
    route
        .any("/test", hello)
        .name("test")
        .middleware(log_middleware::log_request);

    route.group(|todo| {
        todo.name("todos").prefix("/todos");
        todo.get("", todo_controller::index).name("index");
        todo.get("/create", todo_controller::create).name("create");
        todo.post("", todo_controller::store).name("store");
        todo.get("edit/{id}", todo_controller::edit).name("edit");
        todo.post("update/{id}", todo_controller::update)
            .name("update");
        todo.get("delete/{id}", todo_controller::destroy)
            .name("destroy");
    });

    route.build().unwrap_or_else(|e| {
        eprintln!("{:?}", e);
        std::process::exit(1);
    })
}

use axum::extract::State;
use rustavel_core::state::AppState;
// import AppState

async fn hello(State(state): State<AppState>) -> impl IntoResponse {
    let welcome_url = state.route("welcome");
    println!("Welcome route: {}", welcome_url);
    let group_url = state.route("api.v1.users.create");
    println!("Group route: {}", group_url);
    (StatusCode::OK, "hello world")
}
