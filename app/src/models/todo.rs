use rustavel_core::mvc::model::Model;
use serde::{Deserialize, Serialize};
#[derive(Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: u64,
    pub title: String,
    pub done: bool,
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
        &["id", "title", "done"]
    }
}
