use axum::body::Body;
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;

pub async fn log_request(req: Request<Body>, next: Next) -> Response {
    println!("Incoming request: {} {}", req.method(), req.uri());
    next.run(req).await
}
