use rustavel_core::db::schema::Schema;
use crate::migrator::Migration;
use async_trait::async_trait;
use rustavel_core::sql::database_client::DbError;

pub struct CreateTodos;

#[async_trait]
impl Migration for CreateTodos {
    async fn up(&self, schema: &mut Schema) -> Result<(), DbError> {
        schema.create("todos", |table| {
            table.id();
            table.string("title", 127).index().comment("todo title");
            table.boolean("done").default_bool(false).comment("is task done");
            table.timestamps();
        });
        Ok(())
    }

    async fn down(&self, schema: &mut Schema) -> Result<(), DbError> {
        schema.drop_table("todos").await?;
        Ok(())
    }

    fn name(&self) -> &'static str {
        "m_2025_01_15_1945_create_todos"
    }
}