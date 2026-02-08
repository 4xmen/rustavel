use rustavel_core::db::schema::Schema;
use crate::migrator::Migration;
use async_trait::async_trait;
use rustavel_core::sql::database_client::DbError;

pub struct CreateUsers;

#[async_trait]
impl Migration for CreateUsers {
    async fn up(&self, schema: &mut Schema) -> Result<(), DbError> {
        schema.create("users", |table| {
            table.id();
            table.string("name", 127).index().comment("user name");
            table.string("email", 127).unique().comment("user email");
            table.big_integer("parent_id").unsigned().comment("parent id");
            table.boolean("is_dark").default_bool(true).comment("user theme control");
            table.foreign("parent_id").on("users").reference("id").cascade_on_delete();
            table.soft_delete();
            table.timestamps();
            table.validate();
        });
        Ok(())
    }

    async fn down(&self, schema: &mut Schema) -> Result<(), DbError> {
        schema.drop_table("users").await?;
        Ok(())
    }

    fn name(&self) -> &'static str {
        "m_2025_01_15_create_users"
    }
}