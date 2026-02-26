use axum::http::StatusCode;
use axum::response::IntoResponse;
use rustavel_core::routing::BuiltRoutes;
use rustavel_core::routing::Route;

#[derive(Clone, Debug)]
struct Dummy;

#[test]
fn simple_route_test() {
    let mut route: Route<Dummy> = Route::<Dummy>::new();
    route.get("/", sample_handler).name("welcome");
    let builder: BuiltRoutes<Dummy> = route.build().unwrap();
    // very simple test
    assert_eq!(builder.names.get("welcome").unwrap(), "/");
}

#[test]
fn group_route_test() {
    let mut route: Route<Dummy> = Route::<Dummy>::new();
    route.group(|group| {
        group.name("group").prefix("/group");
        group.any("sub", sample_handler).name("sub");
        group.any("/sub2", sample_handler).name("sub2");
    });
    let builder: BuiltRoutes<Dummy> = route.build().unwrap();
    // test / fixer
    assert_ne!(builder.names.get("group.sub").unwrap(), "/groupsub");
    // real test
    assert_eq!(builder.names.get("group.sub2").unwrap(), "/group/sub2");
}

async fn sample_handler() -> impl IntoResponse {
    (StatusCode::OK, "hello world")
}
