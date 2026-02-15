use axum::{extract::State, http::StatusCode, response::IntoResponse};
use axum_params::Params;
use macros::CheckMate;
use rustavel_core::localization::digits::apply_normalized_string;
use rustavel_core::localization::numbers::apply_normalize_number;
use rustavel_core::facades::datetime::{
    deserialize_datetime,
    serialize_datetime
};
use rustavel_core::state::AppState;
use serde::{Deserialize, Serialize};
use time::PrimitiveDateTime;

#[derive(Debug, Serialize, Deserialize, CheckMate)]
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

    #[serde(
        deserialize_with = "deserialize_datetime",
        serialize_with = "serialize_datetime"
    )]
    pub dob: PrimitiveDateTime,
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
