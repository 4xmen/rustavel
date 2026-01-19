use std::sync::Arc;
mod routes;
mod core;
mod app;
mod config;

use crate::core::state::AppState;
use crate::config::CONFIG;

#[tokio::main]
async fn main() {


    let web = routes::web::web_routes();
    let api = routes::api::api_routes();

    let built = web.merge(api).unwrap_or_else(|e| {
        eprintln!("{:?}", e);
        std::process::exit(1);
    });


    let routes_map = Arc::new(built.names.clone());

    println!("route list: {:?}", built.names);

    let state = AppState { routes: routes_map };

    // take type annotation
    let app = built.router.with_state(state);

    let app_start_point = format!(
        "{}:{}",
        CONFIG.app.host,
        CONFIG.app.port
    );



    println!("Starting server on http://{}", app_start_point);
    let listener = tokio::net::TcpListener::bind(app_start_point).await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}