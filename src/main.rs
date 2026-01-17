use std::sync::Arc;
use dotenv::dotenv;
use std::env;

mod routes;
mod core;
mod app;


use crate::core::state::AppState;

#[tokio::main]
async fn main() {

    dotenv().unwrap_or_else(|e|{
        println!(".env file not found, ERROR: {}", e);
        std::process::exit(1);
    });


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

    let app_start_point  = format!("{}:{}",
                                   env::var("APP_IP").ok().unwrap(),
                                   env::var("APP_PORT").ok().unwrap());

    println!("Starting server on http://{}", app_start_point);
    let listener = tokio::net::TcpListener::bind(app_start_point).await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}