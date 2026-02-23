use rustavel_core::mvc::model::Model;
use serde::{Deserialize, Serialize};
use time::PrimitiveDateTime;
#[derive(Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Todo {
    pub id: u64,
    pub title: String,
    pub done: bool,
    pub created_at: PrimitiveDateTime,
    pub updated_at: PrimitiveDateTime,
}

impl Model for Todo {
    type PrimaryKey = u64;

    fn table() -> &'static str {
        "todos"
    }
    fn primary_key() -> &'static str {
        "id"
    }
    fn columns() -> &'static [&'static str] {
        &["id", "title", "done", "created_at", "updated_at"]
    }
}

