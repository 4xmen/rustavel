// Bugs 1 and 2: name()/middleware() inside a group
use axum::{body::Body, extract::Request, http::StatusCode, middleware::Next, response::{IntoResponse, Response}};
use rustavel_core::routing::Route;
use tower::ServiceExt;

async fn deny(_req: Request<Body>, _next: Next) -> Response {
    (StatusCode::FORBIDDEN, "blocked").into_response()
}
async fn ok() -> &'static str { "ok" }

async fn status(router: axum::Router, path: &str) -> u16 {
    let req = Request::get(path).body(Body::empty()).unwrap();
    router.oneshot(req).await.unwrap().status().as_u16()
}

#[tokio::test]
async fn bug1_route_middleware_inside_group_applies_to_that_route_only() {
    let mut r: Route<()> = Route::new();
    r.group(|g| {
        g.prefix("/admin");
        g.get("/secret", ok).name("secret").middleware(deny);
        g.get("/public", ok).name("public");
    });
    let built = r.build().unwrap();
    assert_eq!(status(built.router.clone(), "/admin/secret").await, 403, "/admin/secret should be blocked");
    assert_eq!(status(built.router, "/admin/public").await, 200, "/admin/public should be open");
}

#[tokio::test]
async fn bug1_group_middleware_still_applies_to_the_whole_group() {
    let mut r: Route<()> = Route::new();
    r.group(|g| {
        g.prefix("/admin").middleware(deny);
        g.get("/a", ok);
        g.get("/b", ok);
    });
    let built = r.build().unwrap();
    assert_eq!(status(built.router.clone(), "/admin/a").await, 403);
    assert_eq!(status(built.router, "/admin/b").await, 403);
}

#[tokio::test]
async fn bug2_group_name_is_not_given_to_the_previous_route() {
    let mut r: Route<()> = Route::new();
    r.get("/home", ok);
    r.group(|g| {
        g.name("admin").prefix("/admin");
        g.get("/dash", ok).name("dash");
    });
    let built = r.build().unwrap();
    assert_eq!(built.names.get("admin.dash").map(String::as_str), Some("/admin/dash"));
    assert!(!built.names.contains_key("admin"), "the group name was attached to /home");
}