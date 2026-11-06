use crate::migrator::Migration;
use async_trait::async_trait;
use rustavel_core::db::schema::Schema;
use rustavel_core::sql::database_client::DbError;

pub struct UserCreate;

#[async_trait]
impl Migration for UserCreate {
    async fn up(&self, schema: &mut Schema) -> Result<(), DbError> {
        schema.create("users", |table| {
            table.id();
            table.string("name",128).comment("user name");
            table.string("email",128).unique().comment("user email");
            table.timestamp("email_verified_at").nullable().default_null();
            table.string("password",255);
            table.string("remember_token",255).nullable().default_null();
            table.timestamps();
            table.soft_delete();
            // validate table after create
            let warnings = table.validate();
            for w in warnings {
                eprintln!("warning: {}", w);
            }
        });

        Ok(())
    }

    async fn down(&self, schema: &mut Schema) -> Result<(), DbError> {
        schema.drop_table("users").await?;

        Ok(())
    }

    fn name(&self) -> &'static str {
        "m_2026_11_06_190654_user_create"
    }
}
