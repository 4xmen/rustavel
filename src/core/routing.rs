//! Module for custom routing system inspired by Laravel.
//!
//! Provides a fluent API for defining routes and building Axum routers.
//!
//! # Usage
//! See [`Route`] for examples.

use axum::middleware::Next;
use axum::Router;
use axum::body::Body;
use axum::extract::Request;
use axum::handler::Handler;
use axum::response::Response;
use axum::routing::{MethodRouter, delete, get, options, patch, post, put};
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;

type Middleware<S> = Arc<dyn Fn(MethodRouter<S>) -> MethodRouter<S> + Send + Sync + 'static>;

/// Used internally to map DSL methods like `get` to Axum handlers.
#[allow(dead_code)]
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
    methods: MethodSet,
    middlewares: Vec<Middleware<S>>,
    contexts: Vec<RouteContext<S>>,
}


/// route context hold parent group meta
struct RouteContext<S> {
    path_prefix: String,
    name_prefix: String,
    middlewares: Vec<Middleware<S>>,
}




impl<S> RouteContext<S> {
    fn new() -> Self {
        Self {
            path_prefix: String::new(),
            name_prefix: String::new(),
            middlewares: vec![],
        }
    }
}

// laravel-like route collector
/// Represents a collection of routes, similar to Laravel's router.
///
/// This struct allows building routes in a fluent manner and compiling them into an Axum router.
///
/// # Fields
/// * `items` - A vector of individual route entries (private, so not documented publicly).
/// * `ctx_stack` group content stack
pub struct Route<S = ()> {
    items: Vec<RouteItem<S>>,
    ctx_stack: Vec<RouteContext<S>>,
}

pub struct BuiltRoutes<S = ()> {
    pub router: Router<S>,
    pub names: HashMap<String, String>,
}

/// Route errors enum
#[derive(Debug)]
pub enum RouteError {
    DuplicateRoute {
        name: String,
    },
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


#[allow(dead_code)]
impl<S: Clone + Send + Sync + 'static> Route<S> {
    pub fn new() -> Self {
        Self {
            items: vec![],
            ctx_stack: vec![],
        }
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
            contexts: self.ctx_stack.clone(),
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
    /// Adds a name to current route or group
    ///
    /// # Arguments
    /// * `path` - The URL path for the route or group prefix.
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
    pub fn name(&mut self, name: &str) -> &mut Self {
        if let Some(last) = self.items.last_mut() {
            // If the last route has no name yet and was added
            if last.name.is_empty() {
                last.name = name.to_string();
                return self;
            }
        }

        // Otherwise apply to current group context
        if let Some(ctx) = self.ctx_stack.last_mut() {
            ctx.name_prefix = name.to_string();
        }

        self
    }


    /// Attaches one middleware to the last added route or group.
    ///
    /// Adds a middleware function to the most recently added route or group. The middleware
    /// receives the request and a `Next` handler and must return a `Response`.
    /// Use this to run preprocessing, authentication, logging, etc., for a single
    /// route without affecting other routes or group and apply sub routes of group.
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
        let mw = Arc::new(move |router: MethodRouter<S>| {
            router.layer(axum::middleware::from_fn(mw.clone()))
        });

        if let Some(ctx) = self.ctx_stack.last_mut() {
            ctx.middlewares.push(mw);
        } else if let Some(last) = self.items.last_mut() {
            last.middlewares.push(mw);
        }

        self
    }

    /// Consumes the route collector and builds the final Axum router.
    ///
    /// This method performs three distinct phases:
    ///
    /// 1. **Validation**
    ///    - Detects duplicate routes based on `full_path + method`.
    ///    - Detects duplicate route names *after* group name prefixes are applied.
    ///
    /// 2. **Resolution**
    ///    - Resolves full route paths by applying all group path prefixes.
    ///    - Resolves full route names by applying all group name prefixes.
    ///
    /// 3. **Assembly**
    ///    - Applies group-level and route-level middlewares.
    ///    - Registers routes into the Axum `Router`.
    ///
    /// # Returns
    /// - `Ok(BuiltRoutes<S>)` on success.
    /// - `Err(RouteError)` if a duplicate route or name is detected.
    ///
    /// # Errors
    /// - `RouteError::DuplicateRoute` if two routes resolve to the same path
    ///   and share at least one HTTP method.
    /// - `RouteError::DuplicateName` if two routes resolve to the same full name.
    ///
    /// # Examples
    /// ```rust
    /// let mut route = Route::new();
    /// route.get("/", hello).name("welcome");
    /// let built = route.build().unwrap();
    /// ```
    pub fn build(self) -> Result<BuiltRoutes<S>, RouteError> {
        use std::collections::HashMap;

        /* -------------------------------------------------------------
         * Phase 1: Validation (path + method)
         * -----------------------------------------------------------*/

        let mut seen_routes: HashMap<String, MethodSet> = HashMap::new();

        for item in &self.items {
            let full_path = Self::build_full_path(item);

            let entry = seen_routes
                .entry(full_path.clone())
                .or_insert(MethodSet(0));

            if entry.intersects(item.methods) {
                return Err(RouteError::DuplicateRoute {
                    name: format!("Duplicate route for path: {}", full_path),
                });
            }

            *entry = MethodSet(entry.0 | item.methods.0);
        }

        /* -------------------------------------------------------------
         * Phase 2: Assembly
         * -----------------------------------------------------------*/

        let mut router: Router<S> = Router::new();
        let mut names: HashMap<String, String> = HashMap::new();

        for item in self.items {
            let full_path = Self::build_full_path(&item);
            let full_name = Self::build_full_name(&item);

            // Apply middlewares (group first, then route)
            let mut method_router = item.router;

            for ctx in &item.contexts {
                for mw in &ctx.middlewares {
                    method_router = mw(method_router);
                }
            }

            for mw in &item.middlewares {
                method_router = mw(method_router);
            }

            router = router.route(&full_path, method_router);

            // Register route name (after full resolution)
            if let Some(name) = full_name {
                if names.contains_key(&name) {
                    return Err(RouteError::DuplicateRoute {
                        name: format!("Duplicate route name: {}", name),
                    });
                }
                names.insert(name, full_path);
            }
        }

        Ok(BuiltRoutes { router, names })
    }

    /// Builds the final resolved path for a route by applying all group path prefixes.
    ///
    /// This function concatenates:
    /// - All `path_prefix` values from the route's context stack (in order),
    /// - Followed by the route's own local path.
    ///
    /// The resulting path is normalized to always start with a `/`.
    ///
    /// # Behavior
    /// - Group prefixes are applied in the order they were declared.
    /// - The route's own path is appended last.
    /// - If the final path does not start with `/`, it is automatically inserted.
    ///
    /// # Examples
    /// ```text
    /// Group prefix: "/api"
    /// Route path:   "/users"
    /// Result:       "/api/users"
    ///
    /// Group prefix: "/api"
    /// Route path:   "users"
    /// Result:       "/api/users"
    /// ```
    ///
    /// # Notes
    /// This function does not perform validation or deduplication.
    /// It is intended to be used during the build phase after all
    /// route contexts have been resolved.

    fn build_full_path(item: &RouteItem<S>) -> String {
        let mut path = String::new();

        for ctx in &item.contexts {
            path.push_str(&ctx.path_prefix);
        }

        path.push_str(&item.path);

        if !path.starts_with('/') {
            path.insert(0, '/');
        }

        path
    }

    /// Builds the final resolved route name by applying all group name prefixes.
    ///
    /// This function constructs a fully-qualified route name using dot (`.`)
    /// notation, similar to Laravel route naming.
    ///
    /// # Behavior
    /// - If the route has no local name, `None` is returned.
    /// - Group name prefixes are applied in declaration order.
    /// - Prefixes and the route name are joined using `.`.
    /// - Empty prefixes are ignored.
    ///
    /// # Examples
    /// ```text
    /// Group name: "api"
    /// Route name: "users"
    /// Result:     "api.users"
    ///
    /// Nested groups:
    ///   Group: "api"
    ///   Group: "admin"
    ///   Route: "dashboard"
    /// Result: "api.admin.dashboard"
    /// ```
    ///
    /// # Returns
    /// - `Some(String)` containing the fully-qualified name if the route is named.
    /// - `None` if the route has no name.
    ///
    /// # Notes
    /// Name resolution must occur before duplicate name detection,
    /// since group prefixes affect the final name.
    fn build_full_name(item: &RouteItem<S>) -> Option<String> {
        if item.name.is_empty() {
            return None;
        }

        let mut name = String::new();

        for ctx in &item.contexts {
            if !ctx.name_prefix.is_empty() {
                if !name.is_empty() {
                    name.push('.');
                }
                name.push_str(&ctx.name_prefix);
            }
        }

        if !name.is_empty() {
            name.push('.');
        }

        name.push_str(&item.name);

        Some(name)
    }



    /// Sets a path prefix for the current route group.
    ///
    /// The prefix is applied to all routes declared within the current
    /// group context and any nested groups.
    ///
    /// # Arguments
    /// * `prefix` - A path segment to prepend to all route paths
    ///   in the current group.
    ///
    /// # Behavior
    /// - The prefix is stored in the active `RouteContext`.
    /// - Multiple prefixes are concatenated in declaration order.
    /// - No normalization is performed at this stage.
    ///
    /// # Examples
    /// ```rust
    /// route.group(|r| {
    ///     r.prefix("/api");
    ///     r.get("/users", handler);
    /// });
    /// // Results in "/api/users"
    /// ```
    ///
    /// # Notes
    /// Prefix resolution is deferred until the `build` phase.
    pub fn prefix(&mut self, prefix: &str) -> &mut Self {
        if let Some(ctx) = self.ctx_stack.last_mut() {
            ctx.path_prefix.push_str(prefix);
        }
        self
    }


    /// Creates a route group with its own contextual configuration.
    ///
    /// This method establishes a new routing context that can apply:
    /// - Path prefixes
    /// - Name prefixes
    /// - Middlewares
    ///
    /// to all routes defined inside the provided closure.
    ///
    /// Contexts can be nested, and each nested group inherits and extends
    /// the configuration of its parent.
    ///
    /// # Arguments
    /// * `f` - A closure that receives a mutable reference to the router,
    ///   allowing routes and group configuration to be defined.
    ///
    /// # Behavior
    /// - Pushes a new `RouteContext` onto the context stack.
    /// - Executes the closure within that context.
    /// - Pops the context after the closure completes.
    ///
    /// # Examples
    /// ```rust
    /// route.group(|r| {
    ///     r.prefix("/api").name("api");
    ///
    ///     r.get("/users", list_users).name("users");
    /// });
    /// ```
    ///
    /// # Notes
    /// This method does not immediately register routes.
    /// All resolution occurs during the `build` phase.
    pub fn group<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self),
    {
        self.ctx_stack.push(RouteContext::new());

        f(self);

        self.ctx_stack.pop();
    }
}

impl<S> BuiltRoutes<S>
where
    S: Clone + Send + Sync + 'static,
{
    /// Merges another `BuiltRoutes` instance into this one.
    ///
    /// This method combines:
    /// - The underlying Axum routers using `Router::merge`.
    /// - The route name maps into a single namespace.
    ///
    /// # Type Parameters
    /// * `S` - Shared application state type.
    ///
    /// # Requirements
    /// The state type `S` must implement:
    /// - `Clone`
    /// - `Send`
    /// - `Sync`
    /// - `'static`
    ///
    /// These bounds are required by Axum when merging routers.
    ///
    /// # Behavior
    /// - Routes from `other` are merged into `self`.
    /// - Route names from `other` are inserted into `self.names`.
    /// - Duplicate route names will cause a panic.
    ///
    /// # Examples
    /// ```rust
    /// let web = web_routes();
    /// let api = api_routes();
    ///
    /// let app = web.merge(api);
    /// ```
    ///
    /// # Panics
    /// Panics if a duplicate route name exists between the two route sets.
    ///
    /// # Notes
    /// This method consumes both `BuiltRoutes` instances and returns
    /// a new merged instance.
    pub fn merge(mut self, other: BuiltRoutes<S>) -> Self {
        self.router = self.router.merge(other.router);

        for (name, path) in other.names {
            if self.names.contains_key(&name) {
                panic!("Duplicate route name: {}", name);
            }
            self.names.insert(name, path);
        }

        self
    }
}



impl<S> Clone for RouteContext<S> {
    fn clone(&self) -> Self {
        Self {
            path_prefix: self.path_prefix.clone(),
            name_prefix: self.name_prefix.clone(),
            middlewares: self.middlewares.clone(),
        }
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
        Self(Self::GET | Self::POST | Self::PUT | Self::PATCH | Self::DELETE | Self::OPTIONS)
    }

    fn intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }
}
