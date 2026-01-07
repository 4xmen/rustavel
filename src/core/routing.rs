use axum::Router;
use axum::routing::{delete, get, options, patch, post, put};

#[derive(Debug)]
enum RouteMethod {
    Get,
    Post,
    Put,
    Patch,
    Options,
    Delete,
    Any,
}

impl RouteMethod {
    fn apply(self, router: Router, path: &str) -> Router {
        match self {
            RouteMethod::Get =>
                router.route(path, get(|| async { "Hello world get" })),

            RouteMethod::Post =>
                router.route(path, post(|| async { "Hello world post" })),

            RouteMethod::Put =>
                router.route(path, put(|| async { "Hello world put" })),

            RouteMethod::Patch =>
                router.route(path, patch(|| async { "Hello world patch" })),

            RouteMethod::Options =>
                router.route(path, options(|| async { "Hello world options" })),

            RouteMethod::Delete =>
                router.route(path, delete(|| async { "Hello world delete" })),

            RouteMethod::Any => {
                let router = router
                    .route(path, get(|| async { "Hello world any" }))
                    .route(path, post(|| async { "Hello world any" }))
                    .route(path, put(|| async { "Hello world any" }))
                    .route(path, patch(|| async { "Hello world any" }))
                    .route(path, options(|| async { "Hello world any" }));
                router.route(path, delete(|| async { "Hello world any" }))
            }
        }
    }
}


struct RouteItem {
    path: String,
    method: RouteMethod,
    name: String,
}
pub struct Route {
    items: Vec<RouteItem>,
}

impl Route {
    fn new() -> Route {
        Self {
            items: vec![],
        }
    }

    fn add(&mut self, method: RouteMethod, path: &str) -> &mut Self {
        self.items.push(RouteItem {
            path: path.to_string(),
            method,
            name: String::new(),
        });
        self
    }

    pub fn get(&mut self, path: &str) -> &mut Self {
        self.add(RouteMethod::Get, path)
    }

    pub fn post(&mut self, path: &str) -> &mut Self {
        self.add(RouteMethod::Post, path)
    }

    pub fn put(&mut self, path: &str) -> &mut Self {
        self.add(RouteMethod::Put, path)
    }

    pub fn patch(&mut self, path: &str) -> &mut Self {
        self.add(RouteMethod::Patch, path)
    }

    pub fn delete(&mut self, path: &str) -> &mut Self {
        self.add(RouteMethod::Delete, path)
    }

    pub fn options(&mut self, path: &str) -> &mut Self {
        self.add(RouteMethod::Options, path)
    }

    pub fn any(&mut self, path: &str) -> &mut Self {
        self.add(RouteMethod::Any, path)
    }

    pub fn name(&mut self, route_name: &str) -> &mut Self {
        if let Some(last) = self.items.last_mut() {
            last.name = route_name.to_string();
        }
        self
    }
}