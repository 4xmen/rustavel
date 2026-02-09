
use std::time::Instant;
use crate::migrations::get_all_migrations;
use async_trait::async_trait;
use rustavel_core::db::schema::Schema;
use rustavel_core::facades::terminal_ui::{*};
use rustavel_core::sql::database_client::DbError;

#[async_trait]
pub trait Migration: Send + Sync {
    async fn up(&self, schema: &mut Schema) -> Result<(), DbError>;
    async fn down(&self, schema: &mut Schema) -> Result<(), DbError>;
    fn name(&self) -> &'static str;
}

pub async fn run_migrations(up: bool, passive: bool, fresh: bool) -> Result<(), DbError> {
    let migrations = get_all_migrations();
    let mut batch = 1;
    let mut schema = Schema::new().await?;
    let mut migrated_count = 0;
    let mut start = Instant::now();
    let migration_list : Vec<String> =  if !passive {
        if fresh {
            start = Instant::now();
            schema.drop_all_tables().await?;
            operation( "Dropping all tables" ,start.elapsed(),Status::Done)
        }
        // check migration table
        if !schema.repository_exists().await? {
            start = Instant::now();
            title(TitleKind::Info,"Preparing database.");
            schema.create_migration_table().await?;
        }
        batch = schema.get_next_batch_number().await?;
        schema.get_migrations_listing().await?
    } else {
        vec![]
    };


    title(TitleKind::Info,"Running migrations.");
    for mig in migrations {
        if up {
            mig.up(&mut schema).await?;
            if !passive && !migration_list.contains(&mig.name().to_string()) {
                // run migration
                schema.execute_migration(mig.name(),&start.into()).await?;
                // add to table
                schema.add_migrated_table(mig.name(),batch).await?;

                migrated_count += 1;

            }
        } else {
            mig.down(&mut schema).await?;
        }
    }

    if migrated_count == 0 {
        title(TitleKind::Info,"Noting to migrate");
    }
    Ok(())
}
