use axum::extract::{State,RawPathParams,Request,RawQuery};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use crate::core::schema::Schema;
use crate::core::state::AppState;



pub async fn index(State(state): State<AppState>) -> impl IntoResponse {
    Schema::create("users",|table|{
        table.comment("users table comment");
        table.id();
        table.string("name",127).index().comment("user name");
        table.string("email",127).unique().comment("user email");
        table.enums("role",vec!["admin".to_string(),"user".to_string()]);
        table.soft_delete();
        table.timestamps();
        dbg!("{:?}",table);
    });
    println!("test columns {:?}", Schema::get_columns("users"));
    println!("test tables {:?}", Schema::get_tables());
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
