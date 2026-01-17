use axum::extract::{State,RawPathParams,Request,RawQuery};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::core::state::AppState;



pub async fn index(State(state): State<AppState>) -> impl IntoResponse {
    (StatusCode::OK, "to index called")
}
pub async fn create(State(state): State<AppState>) -> impl IntoResponse {
    (StatusCode::OK, "to create called")
}
pub async fn store(State(state): State<AppState>) -> impl IntoResponse {
    (StatusCode::OK, "to store called")
}
pub async fn edit(State(state): State<AppState>, params: RawPathParams) -> impl IntoResponse {
    (StatusCode::OK, println!("to edit called id: {:?}",params))
}
pub async fn update(State(state): State<AppState>, params: RawPathParams) -> impl IntoResponse {
    (StatusCode::OK, println!("to  update called id: {:?}",params))
}
pub async fn destroy(State(state): State<AppState>, params: RawPathParams,query: RawQuery) -> impl IntoResponse {
    // for (key, value) in &params {
    //     println!("{key:?} = {value:?}");
    // }
    (StatusCode::OK, println!("to destroy called id:  {:?}, {:?}, {:?}",state,query,params))
}
