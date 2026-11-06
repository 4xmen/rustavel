use axum::Router;
use tower_http::services::ServeDir;

pub fn static_files<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    let storage = ServeDir::new("storage/app/public");
    let public = ServeDir::new("public");

    Router::new()
        .nest_service("/storage", storage)
        .fallback_service(public)
}