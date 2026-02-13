use axum::{extract::State, http::StatusCode, response::IntoResponse, };
use axum_params::Params;
use serde::{Deserialize, Serialize};
use rustavel_core::state::AppState;
use macros::CheckMate;
use rustavel_core::localization::numbers::apply_normalize_number;
use rustavel_core::localization::digits::apply_normalized_string;


#[derive(Debug, Serialize, Deserialize,CheckMate)]
pub struct RegPayload {
    #[validating("required|email|min:2|max:180")]
    pub title: String,
    pub description: String,
    #[serde(deserialize_with = "apply_normalized_string")]
    pub code: String,
    pub path: String,
    #[serde(deserialize_with = "apply_normalize_number")]
    #[validating("required|min:17|max:180")]
    pub age: i64,
}




#[axum::debug_handler]
pub async fn register(
    State(_state): State<AppState>,
    Params(payload, _): Params<RegPayload>,
) -> impl IntoResponse {
    let x = payload.validate().await;
    println!("{:#?}", x);

    (StatusCode::OK, "hello world")
}

