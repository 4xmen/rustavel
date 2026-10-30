use std::time::Instant;
use axum::{extract::State, response::IntoResponse, Extension};
// use minijinja::functions::debug;
use serde::Serialize;
use rustavel_core::config::CONFIG;
use rustavel_core::helper::datetime::format_duration;
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
    
    state.view("welcome.j2", Some(sample))
}
