use axum::{extract::State, http::StatusCode, response::IntoResponse};
use axum_params::Params;
use macros::CheckMate;
use rustavel_core::localization::digits::apply_normalized_string;
// use rustavel_core::localization::numbers::apply_normalize_number;
use rustavel_core::facades::datetime::*;
use rustavel_core::state::AppState;
use serde::{Deserialize, Serialize};
use time::{Date, PrimitiveDateTime};
// use time::Date;
// use time::Time;

#[derive(Debug, Serialize, Deserialize, CheckMate)]
pub struct RegPayload {
    #[validating("required|email|min:12|max:180")]
    pub title: String,
    pub description: String,
    #[serde(deserialize_with = "apply_normalized_string")]
    #[validating("required|alphanumeric")]
    pub code: String,
    pub path: String,
    // #[serde(deserialize_with = "apply_normalize_number")]
    #[validating("required|min:18|max:900|size:2")]
    pub age: i64,

    #[serde(
        deserialize_with = "deserialize_date",
        serialize_with = "serialize_date"
    )]
    #[validating("required|date|after:2026-01-01")]
    pub dob: Date,
    #[serde(
        deserialize_with = "deserialize_datetime",
        serialize_with = "serialize_datetime"
    )]
    #[validating("required|datetime|after:2020-01-01")]
    pub published: PrimitiveDateTime,

    #[validating("nullable|date|after:2020-01-01|before:2000-03-12")]
    pub omg: Option<String>,

    #[validating("nullable|min:10|confirmed:pass_confirm")]
    pub pass: Option<String>,
    pub pass_confirm: Option<String>,

}

#[axum::debug_handler]
pub async fn register(
    State(_state): State<AppState>,
    Params(payload, _): Params<RegPayload>,
) -> impl IntoResponse {
    let x = payload.validate().await;
    println!("{:#?}", x);

    (StatusCode::OK, "hello world2")
}
