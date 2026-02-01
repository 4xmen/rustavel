use std::vec;
use axum::extract::{RawPathParams, RawQuery, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse};
use rustavel_core::state::AppState;
use crate::models::todo::Todo;
use axum::Json;
use rustavel_core::sql::query::QueryDsl;
pub async fn index(State(_state): State<AppState>) -> impl IntoResponse {
    // just for test now
    let tasks =  vec!(
        Todo{
            id: 1,
            title: "create router".to_string(),
            done: true,
        },
        Todo{
            id: 2,
            title: "create schema".to_string(),
            done: true,
        },
        Todo{
            id: 3,
            title: "create controller".to_string(),
            done: false,
        },

    );
    (StatusCode::OK, Json(tasks))
}
pub async fn create(State(_state): State<AppState>) -> impl IntoResponse {
    println!("try to call all");
    Todo::all();
    (StatusCode::OK, "to create called")
}
pub async fn store(State(_state): State<AppState>) -> impl IntoResponse {
    (StatusCode::OK, "to store called")
}
pub async fn edit(State(_state): State<AppState>, params: RawPathParams) -> impl IntoResponse {

    (StatusCode::OK, println!("to edit called id: {:?}", params))
}
pub async fn show(State(_state): State<AppState>, params: RawPathParams) -> impl IntoResponse {

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
