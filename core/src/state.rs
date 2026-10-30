use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use minijinja::Environment;
use minijinja::value::Serde;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use axum::extract::FromRef;

#[derive(Clone, Debug, FromRef)]
pub struct AppState {
    pub routes: Arc<HashMap<String, String>>,
    pub templates: Arc<Environment<'static>>,
}

impl AppState {

    /// Get route url by route name return str
    /// if the route exsits return
    /// otherwise return #route-name-not-found
    pub fn route(&self, route: &str) -> &str {
        match self.routes.get(route) {
            Some(route) => route,
            None => "#route-name-not-found",
        }
    }


    /// Renders a MiniJinja template and returns it as an HTML response.
    ///
    /// If `data` is `Some`, it is passed to the template as its context. If `data`
    /// is `None`, the template is rendered with an empty context. Returns `200 OK`
    /// when rendering succeeds, or `500 Internal Server Error` if the template
    /// cannot be found or rendered.
    pub fn view<T>(&self, view_file: &str, data: Option<T>) -> impl IntoResponse  + use<T>
    where
        T: Serialize,
    {
        // Render the template.

        let result = self
            .templates
            .get_template(view_file)
            .and_then(|template| match data {
                Some(data) => template.render(Serde(&data)),
                None => template.render(()),
            });

        match result {
            Ok(html) => (StatusCode::OK, Html(html)).into_response(),
            Err(error) => {
                eprintln!("Template rendering failed: {error}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }

}
