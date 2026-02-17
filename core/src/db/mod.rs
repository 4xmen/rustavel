use crate::db::schema::Schema;
use tokio::sync::OnceCell;

pub mod schema;
pub mod table;

pub static SCHEMA: OnceCell<Schema> = OnceCell::const_new();

pub async fn get_static_schema() -> &'static Schema {
    SCHEMA
        .get_or_init(|| async { Schema::new().await.unwrap() })
        .await
}
