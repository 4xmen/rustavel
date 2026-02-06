use async_trait::async_trait;
use rustavel_core::db::schema::Schema;
use rustavel_core::sql::database_client::{ DbError};
use crate::migrations::get_all_migrations;


#[async_trait]
pub trait Migration: Send + Sync {
    async fn up(&self, schema: &mut Schema) -> Result<(), DbError>;
    async fn down(&self, schema: &mut Schema) -> Result<(), DbError>;
    fn name(&self) -> &'static str;
}

pub async fn run_migrations(up: bool) -> Result<(), DbError> {

    let migrations = get_all_migrations();  // از migrations/mod.rs

    for mig in migrations {
        let mut schema = Schema::new().await?;  // DSL تو
        if up {
            mig.up(&mut schema).await?;
        } else {
            mig.down(&mut schema).await?;
        }
        //  may exec sql
        // track in __migrations table
    }
    Ok(())
}

