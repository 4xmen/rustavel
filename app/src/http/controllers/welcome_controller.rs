use std::time::Instant;
use axum::{extract::State, response::IntoResponse, http::StatusCode, Extension};
use axum::response::Html;
use minijinja::value::Serde;
use serde::Serialize;
use rustavel_core::config::CONFIG;
use rustavel_core::facades::datetime::format_duration;
use rustavel_core::state::AppState;

#[derive(Serialize)]
struct SampleData {
    title: String,
    elapsed_time: Option<String>,
}

/// Renders the welcome page using MiniJinja.
pub async fn index(
    State(state): State<AppState>,
    Extension(start): Extension<Instant>,
) -> impl IntoResponse {
    // Prepare the data passed to the template.


    let elapsed_ms = format_duration(start.elapsed());

    let sample = SampleData {
        title: CONFIG.app.name.clone(),
        elapsed_time: Some(elapsed_ms),
    };
    // Measure template rendering time.
    // let render_start = Instant::now();

    // Render the welcome template.
    let result = state.templates
        .get_template("welcome.j2")
        .and_then(|template| {
            template.render(Serde(&sample))
        });

    // println!(
    //     "[PERF] template render: {}",
    //     format_duration(render_start.elapsed()),
    // );
    match result {
        Ok(html) => (StatusCode::OK, Html(html)).into_response(),
        Err(error) => {
            eprintln!("Template rendering failed: {error}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
