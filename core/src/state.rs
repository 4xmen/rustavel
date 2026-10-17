use std::collections::HashMap;
use std::sync::Arc;
use minijinja::Environment;

#[derive(Clone, Debug)]
pub struct AppState {
    pub routes: Arc<HashMap<String, String>>,
    pub templates: Arc<Environment<'static>>,
}

impl AppState {
    pub fn route(&self, route: &str) -> &str {
        match self.routes.get(route) {
            Some(route) => route,
            None => "#route-name-not-found",
        }
    }
}
