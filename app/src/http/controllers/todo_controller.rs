// use std::vec;
use axum::extract::{RawPathParams, RawQuery, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse};
use rustavel_core::state::AppState;
use axum::Json;
use time::macros::{ datetime};

// use time::PrimitiveDateTime;
// use rustavel_core::sql::query::QueryDsl;
use crate::models::Todo;
use rustavel_core::facades::datetime::now_primitive;
use rustavel_core::facades::datetime::TimeExt;


pub async fn index(State(_state): State<AppState>) -> impl IntoResponse {
    // just for test now
    let tasks =  vec!(
        Todo{
            id: 1,
            title: "create router".to_string(),
            done: true,
            created_at: now_primitive(),
            updated_at: now_primitive(),

        },
        Todo{
            id: 2,
            title: "create schema".to_string(),
            done: true,
            created_at: now_primitive(),
            updated_at: now_primitive(),
        },
        Todo{
            id: 3,
            title: "create controller".to_string(),
            done: false,
            created_at: now_primitive(),
            updated_at: now_primitive(),
        },

    );
    (StatusCode::OK, Json(tasks))
    // (StatusCode::OK, "so so")
}
pub async fn create(State(_state): State<AppState>) -> impl IntoResponse {
    // Todo::all();
    println!("so so...");
    (StatusCode::OK, "to create called")
}
pub async fn store(State(_state): State<AppState>) -> impl IntoResponse {
    (StatusCode::OK, "to store called")
}
pub async fn edit(State(_state): State<AppState>, params: RawPathParams) -> impl IntoResponse {

    (StatusCode::OK, println!("to edit called id: {:?}", params))
}
#[axum::debug_handler]
pub async fn update(State(_state): State<AppState>, params: RawPathParams) -> impl IntoResponse {

    (StatusCode::OK, println!("to edit called id: {:?}", params))
}
pub async fn show(State(_state): State<AppState>, params: RawPathParams) -> impl IntoResponse {

    let x = datetime!(2026-04-12 15:20:20);

    println!("{},{}",x.diff_for_humans(),x.ldate("Y/m/d H:i:s"));
    (StatusCode::OK, format!("to edit called id: {:?},{}", params,x.format_php("Y/m/d H:i:s w")))
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
