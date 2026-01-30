use async_trait::async_trait;
use rustavel_core::schema::Schema;
use rustavel_core::sql::database_client::{DatabaseClient, DbError};
use crate::migrations::m_2025_01_15_1945_create_users::CreateUsers;

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

fn get_all_migrations() -> Vec<Box<dyn Migration>> {
    // may need do it auto mex time
    vec![
        Box::new(CreateUsers {}),
        // add other migration
    ]
}
