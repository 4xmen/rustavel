use std::fmt::format;
use crate::core::schema::Schema;
use crate::core::state::AppState;
use axum::extract::{RawPathParams, RawQuery, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use minijinja::filters::default;
use crate::core::logger;

fn assert_send_val<T: Send>(_: &T) {}

pub async fn index(State(_state): State<AppState>) -> impl IntoResponse {
    // just for test now


    let s = Schema::new().await.unwrap();
    assert_send_val(&s); // check error
    let tables = s.get_column_listing("users").await;
    let tables = s.get_tables().await;
    println!("{:?}",tables);

    (StatusCode::OK, "to index called")
}
pub async fn create(State(_state): State<AppState>) -> impl IntoResponse {

    Schema::new().await.unwrap().create("users", |table| {
        table.table_comment("user table");
        table.id();
        table.string("name", 127).index().comment("user name");
        table.string("email", 127).unique().comment("user email");
        table.big_integer("team_id").unsigned().index().comment("team from table");
        table.big_integer("parent_id").unsigned();
        table.boolean("is_blocked").default_bool(true);
        table.enums("role", vec!["admin", "user"]).default_str("admin");
        table.json("data").default_json_array().comment("data from table json");
        table.soft_delete();
        table.timestamps();
        //
        table.foreign("team_id").on("teams").reference("id").cascade_on_delete();
        table.foreign("parent_id").on("users").reference("id").cascade_on_delete();

        table.validate();


    });
    (StatusCode::OK, "to create called")
}
pub async fn store(State(_state): State<AppState>) -> impl IntoResponse {
    (StatusCode::OK, "to store called")
}
pub async fn edit(State(_state): State<AppState>, params: RawPathParams) -> impl IntoResponse {

    // let mut s = Schema::new().await.unwrap();
    // assert_send_val(&s); // check error if u
    // // println!("{:?}",s.rename("userz","users").await);
    // // println!("{:?}",s.has_index("users",vec!["id","email"]).await);
    //  println!("{:?}",  s.rename_prefix("").await);
    Schema::new().await.unwrap().table("tester",|table|{
       table.string("avatar",1024).collation("utf8mb4_bin").comment("avatar image");
       table.decimal("val",20,2).unsigned().nullable().comment("changed").change();
       table.drop_column("un");
    });

    (StatusCode::OK, println!("to edit called id: {:?}", params))
}
pub async fn update(State(_state): State<AppState>, params: RawPathParams) -> impl IntoResponse {
    (
        StatusCode::OK,
        println!("to  update called id: {:?}", params),
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

