//! Module for custom routing system inspired by Laravel.
//!
//! Provides a fluent API for defining routes and building Axum routers.
//!
//! # Usage
//! See [`Route`] for examples.

use axum::Router;
use axum::handler::Handler;
use axum::routing::{MethodRouter, delete, get, options, patch, post, put};
use std::collections::HashMap;

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

    // build final axum router
    /// Consumes the collector and builds the final Axum router with a name-to-path map.
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
    pub fn build(self) -> BuiltRoutes<S> {
        let mut router: Router<S> = Router::new();
        let mut names = HashMap::new();

        for item in self.items {
            if !item.name.is_empty() {
                names.insert(item.name.clone(), item.path.clone());
            }

            router = router.route(&item.path, item.router);
        }

        BuiltRoutes { router, names }
    }
}
