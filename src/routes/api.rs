
use axum::http::StatusCode;
use axum::response::IntoResponse;
use crate::core::routing::{Route,BuiltRoutes};
use crate::app::http::middleware::log_middleware;
use crate::core::state::AppState; // import AppState



pub fn api_routes() -> BuiltRoutes<AppState> {
    let mut route = Route::new();

    route.group(|r| {
        r.name("api").prefix("/api").middleware(log_middleware::log_request);

        r.group(|v1|{
            v1.prefix("/v1").name("v1");
            v1.get("", hello_api).name("index");
            v1.group(|users| {
                users.prefix("/users").name("users");
                users.get("index", hello_api).name("index");
                users.get("create", hello_api).name("create");
            })
        })
    });

    route.build().unwrap_or_else(|e| {
        eprintln!("{:?}", e);
        std::process::exit(1);
    })
}


async fn hello_api() -> impl IntoResponse {
    (StatusCode::OK, "hello api")
}