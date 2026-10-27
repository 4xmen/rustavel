
use std::path::Path;
use std::sync::Arc;

mod http;
mod models;
mod routes;

use rustavel_core::config::CONFIG;
use rustavel_core::logger;
use rustavel_core::state::AppState;
// use crate::http::controllers::test_controller::register;
// use axum::Router;
// use axum::routing::post;
use axum::extract::DefaultBodyLimit;
use minijinja::{Environment, path_loader};

#[tokio::main]
async fn main() {
    let web = routes::web::web_routes();
    let api = routes::api::api_routes();

    let built = web.merge(api).unwrap_or_else(|e| {
        eprintln!("{:?}", e);
        std::process::exit(1);
    });

    let routes_map = Arc::new(built.names.clone());

    // load minijinja template

    // let views_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
    //     .join("src")
    //     .join("resources")
    //     .join("views");

    // APP_VIEWS_PATH for deployed binaries; the source tree otherwise
    let views_dir = std::env::var("APP_VIEWS_PATH")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("src")
                .join("resources")
                .join("views")
        });

    let mut env = Environment::new();
    env.set_loader(path_loader(views_dir));

    let state = AppState {
        routes: routes_map,
        templates: Arc::new(env),
    };

    // take type annotation
    let app = built
        .router
        .layer(DefaultBodyLimit::max(CONFIG.app.max_upload_size))
        .with_state(state);

    //
    // let app: Router<AppState> = Router::new()
    //     .with_state(state)
    //     .route("/test", post(register));

    let app_start_point = format!("{}:{}", CONFIG.app.host, CONFIG.app.port);

    logger::info(&format!("Try server on http://{}", app_start_point));

    // let listener = tokio::net::TcpListener::bind(app_start_point.clone())
    //     .await
    //     .unwrap();


    let listener = tokio::net::TcpListener::bind(app_start_point.clone())
        .await
        .unwrap_or_else(|e| {
            eprintln!("can't listen on {}: {}", app_start_point, e);
            std::process::exit(1);
        });


    logger::success(&format!("Starting server on http://{}", app_start_point));
    axum::serve(listener, app).await.unwrap();
}
