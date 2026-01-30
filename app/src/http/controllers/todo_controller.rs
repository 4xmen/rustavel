use axum::extract::{RawPathParams, RawQuery, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use rustavel_core::state::AppState;

fn assert_send_val<T: Send>(_: &T) {}

pub async fn index(State(_state): State<AppState>) -> impl IntoResponse {
    // just for test now
    (StatusCode::OK, "to index called")
}
pub async fn create(State(_state): State<AppState>) -> impl IntoResponse {
    (StatusCode::OK, "to create called")
}
pub async fn store(State(_state): State<AppState>) -> impl IntoResponse {
    (StatusCode::OK, "to store called")
}
pub async fn edit(State(_state): State<AppState>, params: RawPathParams) -> impl IntoResponse {

    (StatusCode::OK, println!("to edit called id: {:?}", params))
}
pub async fn update(State(_state): State<AppState>, params: RawPathParams) -> impl IntoResponse {
    (
        StatusCode::OK,
        println!("to update called id: {:?}", params),
    )
}
pub async fn destroy(
    State(_state): State<AppState>,
    params: RawPathParams,
    query: RawQuery,
) -> impl IntoResponse {
    // for (key, value) in &params {
    //     println!("{key:?} = {value:?}");
    // }
    (
        StatusCode::OK,
        println!(
            "to destroy called id:  {:?}, {:?}, {:?}",
            _state, query, params
        ),
    )
}
