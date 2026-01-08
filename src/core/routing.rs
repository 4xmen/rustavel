//! Module for custom routing system inspired by Laravel.
//!
//! Provides a fluent API for defining routes and building Axum routers.
//!
//! # Usage
//! See [`Route`] for examples.

use axum::Router;
use axum::body::Body;
use axum::extract::Request;
use axum::handler::Handler;
use axum::middleware::Next;
use axum::response::Response;
use axum::routing::{MethodRouter, delete, get, options, patch, post, put};
use std::collections::HashMap;
use std::future::Future;
use tower::Layer;
use colored::*;


/// Used internally to map DSL methods like `get` to Axum handlers.
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

// single route entry
pub struct RouteItem<S = ()> {
    path: String,
    name: String,
    router: MethodRouter<S>,
    methods: MethodSet, // for check duplicate route
    middlewares: Vec<
        Box<
            dyn Fn(MethodRouter<S>) -> MethodRouter<S>
            + Send
            + Sync
            + 'static
        >
    >,
}



// laravel-like route collector
/// Represents a collection of routes, similar to Laravel's router.
///
/// This struct allows building routes in a fluent manner and compiling them into an Axum router.
///
/// # Fields
/// * `items` - A vector of individual route entries (private, so not documented publicly).
pub struct Route<S = ()> {
    items: Vec<RouteItem<S>>,
}

pub struct BuiltRoutes<S = ()> {
    pub router: Router<S>,
    pub names: HashMap<String, String>,
}


/// Route errors enum
#[derive(Debug)]
pub enum RouteError {
    DuplicateRoute(String),
}

/// store method set to check on build
#[derive(Clone, Copy, Debug)]
struct MethodSet(u8);

macro_rules! define_method {
    ($method:ident, $enum_variant:ident) => {
        #[doc = concat!("Adds a ", stringify!($enum_variant), " route to the collector.")]
        ///
        /// # Arguments
        #[doc = concat!(" * `path` - The URL path for the ", stringify!($enum_variant), " route.")]
        /// * `handler` - The handler function for this route.
        ///
        /// # Returns
        ///
        /// A mutable reference to self for chaining.
        ///
        /// # Examples
        /// ```
        /// let mut route = Route::new();
        #[doc = concat!("route.", stringify!($method), "(\"/\", hello_handler).name(\"welcome\");")]
        /// ```
        pub fn $method<H, T>(&mut self, path: &str, handler: H) -> &mut Self
        where
            H: Handler<T, S> + Clone + Send + Sync + 'static,
            T: 'static,
        {
            self.add(RouteMethod::$enum_variant, path, handler)
        }
    };
}




impl<S: Clone + Send + Sync + 'static> Route<S> {
    pub fn new() -> Self {
        Self { items: vec![] }
    }

    // internal helper to reduce duplication
    fn add<H, T>(&mut self, method: RouteMethod, path: &str, handler: H) -> &mut Self
    where
        H: Handler<T, S> + Clone + Send + Sync + 'static,
        T: 'static,
    {

        // store method set to check duplicate
        let methods = match method {
            RouteMethod::Get => MethodSet(MethodSet::GET),
            RouteMethod::Post => MethodSet(MethodSet::POST),
            RouteMethod::Patch => MethodSet(MethodSet::PATCH),
            RouteMethod::Delete => MethodSet(MethodSet::DELETE),
            RouteMethod::Options => MethodSet(MethodSet::OPTIONS),
            RouteMethod::Put => MethodSet(MethodSet::PUT),
            RouteMethod::Any => MethodSet::any(),
        };

        // convert method + handler into axum method router
        let router = match method {
            RouteMethod::Get => get(handler),
            RouteMethod::Post => post(handler),
            RouteMethod::Put => put(handler),
            RouteMethod::Patch => patch(handler),
            RouteMethod::Options => options(handler),
            RouteMethod::Delete => delete(handler),
            RouteMethod::Any => get(handler.clone())
                .post(handler.clone())
                .put(handler.clone())
                .patch(handler.clone())
                .options(handler.clone())
                .delete(handler),
        };

        self.items.push(RouteItem {
            path: path.to_string(),
            name: String::new(),
            router,
            middlewares: vec![],
            methods,
        });

        self
    }

    // public dsl methods

    define_method!(get, Get);
    define_method!(post, Post);
    define_method!(put, Put);
    define_method!(patch, Patch);
    define_method!(options, Options);
    define_method!(delete, Delete);

    /// Adds GET,POST,PUT,PATCH,DELETE,OPTIONS route to the collector.
    ///
    /// # Arguments
    /// * `path` - The URL path for the route.
    /// * `handler` - The handler function for this route. Used to remove resources; handlers should return appropriate status codes (e.g., 204 No Content on success).
    ///
    /// # Returns
    /// A mutable reference to self for chaining.
    ///
    /// # Examples
    /// ```
    /// let mut route = Route::new();
    /// route.any("/", index).name("index");
    /// ```
    pub fn any<H, T>(&mut self, path: &str, handler: H) -> &mut Self
    where
        H: Handler<T, S> + Clone + Send + Sync + 'static,
        T: 'static,
    {
        self.add(RouteMethod::Any, path, handler)
    }

    // attach name to last route
    /// Adds a name to current route
    ///
    /// # Arguments
    /// * `path` - The URL path for the route.
    /// * `handler` - The handler function for this route. Use for every method
    ///
    /// # Returns
    /// A mutable reference to self for chaining.
    ///
    /// # Examples
    /// ```
    /// let mut route = Route::new();
    /// route.get("/users", user_list).name("user.index");
    /// ```
    pub fn name(&mut self, route_name: &str) -> &mut Self {
        if let Some(last) = self.items.last_mut() {
            last.name = route_name.to_string();
        }
        self
    }

    /// Attaches one middleware to the last added route.
    ///
    /// Adds a middleware function to the most recently added route. The middleware
    /// receives the request and a `Next` handler and must return a `Response`.
    /// Use this to run preprocessing, authentication, logging, etc., for a single
    /// route without affecting other routes.
    ///
    /// # Type parameters
    /// * `F` - Middleware function type: `Fn(Request<Body>, Next) -> Fut + Clone + Send + Sync + 'static`.
    /// * `Fut` - The future returned by the middleware: `Future<Output = Response> + Send + 'static`.
    ///
    /// # Arguments
    /// * `mw` - The middleware function to attach. The function will be cloned when
    ///   added so it must implement `Clone`.
    ///
    /// # Behavior
    /// * If there is no previously added route, this method is a no-op.
    /// * Routes without middleware keep an empty `Vec`, so attaching middleware
    ///   imposes minimal overhead.
    ///
    /// # Returns
    /// A mutable reference to self for chaining.
    ///
    /// # Examples
    /// ```
    /// let mw = |req: Request<Body>, next: Next| async move {
    ///     // e.g., authentication or logging
    ///     next.run(req).await
    /// };
    ///
    /// let mut route = Route::new();
    /// route.post("/login", login_handler).middleware(mw);
    /// ```
    pub fn middleware<F, Fut>(&mut self, mw: F) -> &mut Self
    where
        F: Fn(Request<Body>, Next) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        if let Some(last) = self.items.last_mut() {
            last.middlewares.push(Box::new(move |router| {
                router.layer(axum::middleware::from_fn(mw.clone()))
            }));
        }
        self
    }


    // build final axum router
    /// Consumes the collector and builds the final Axum router with a name-to-path map and add middleware if available
    ///
    /// # Returns
    /// A `BuiltRoutes<S>` containing:
    /// - `router`: the assembled `Router<S>` with all registered routes.
    /// - `names`: a `HashMap<String, String>` mapping route names to their paths (only entries for routes with non-empty names).
    ///
    /// # Behavior
    /// - Consumes `self`, taking ownership of the collected route items.
    /// - Registers each item's route into the router with `router.route(&item.path, item.router)`.
    /// - If `item.name` is not empty, inserts `item.name.clone()` -> `item.path.clone()` into `names`.
    ///
    /// # Examples
    /// ```
    /// let mut route = Route::new();
    /// route.get("/", hello_handler).name("welcome");
    /// let built: BuiltRoutes<AppState> = route.build();
    /// let routes_map = Arc::new(built.names.clone());
    /// let state = AppState { routes: routes_map };
    /// let app = built.router.with_state(state);
    /// ```
    pub fn build(self) -> Result<BuiltRoutes<S>, RouteError>  {

    for i in 0..self.items.len() {
            for j in (i + 1)..self.items.len() {
                let a = &self.items[i];
                let b = &self.items[j];

                if a.path == b.path && a.methods.intersects(b.methods) {
                    return Err(RouteError::DuplicateRoute(format!("Duplicate route: {} methods", a.path)));
                }
            }
        }

        let mut router: Router<S> = Router::new();
        let mut names = HashMap::new();

        for item in self.items {
            if !item.name.is_empty() {
                // check duplicate route name
                if names.contains_key(&item.name) {
                    println!("{}", format!("Warning: Duplicate route name '{}'", item.name).yellow());
                }else{
                    names.insert(item.name.clone(), item.path.clone());
                }
            }

            let mut method_router = item.router;

            for mw in item.middlewares {
                method_router = mw(method_router);
            }


            router = router.route(&item.path, method_router);
        }

        Ok(BuiltRoutes { router, names })

    }
}

// add const and any function
impl MethodSet {
    const GET: u8 = 1 << 0;
    const POST: u8 = 1 << 1;
    const PUT: u8 = 1 << 2;
    const PATCH: u8 = 1 << 3;
    const DELETE: u8 = 1 << 4;
    const OPTIONS: u8 = 1 << 5;

    fn any() -> Self {
        Self(
            Self::GET
                | Self::POST
                | Self::PUT
                | Self::PATCH
                | Self::DELETE
                | Self::OPTIONS,
        )
    }

    fn intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }
}