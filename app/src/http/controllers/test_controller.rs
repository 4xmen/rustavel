use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::post, Router};
use axum_params::Params;
use serde::{Deserialize, Serialize};
use rustavel_core::state::AppState;


#[derive(Debug, Serialize, Deserialize)]
pub struct RegPayload {
    pub title: String,
    pub description: String,
    pub code: String,
    pub path: String,
    pub age: i64,
}

impl RegPayload {
    async fn is_valid(&self) {
        println!("{:?}", self);
    }
}


#[axum::debug_handler]
pub async fn register(
    State(_state): State<AppState>,
    Params(payload, _): Params<RegPayload>,
) -> impl IntoResponse {
    payload.is_valid().await;

    (StatusCode::OK, "hello world")
}

