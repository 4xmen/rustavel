use crate::migrations::get_all_migrations;
use async_trait::async_trait;
use illuminate_str::Str;
use rustavel_artisan::general::generate_model::save_generated_model;
use rustavel_core::db::schema::Schema;
use rustavel_core::facades::terminal_ui::*;
use rustavel_core::sql::database_client::DbError;
use std::time::Instant;
#[async_trait]
pub trait Migration: Send + Sync {
    async fn up(&self, schema: &mut Schema) -> Result<(), DbError>;
    async fn down(&self, schema: &mut Schema) -> Result<(), DbError>;
    fn name(&self) -> &'static str;
}

pub async fn run_migrations(rollback: i64, passive: bool, fresh: bool) -> Result<(), DbError> {
    let migrations = get_all_migrations();
    let mut batch = 1;
    let mut schema = Schema::new().await?;
    let mut migrated_count = 0;
    let migration_list: Vec<String> = if !passive {
        if fresh {
            let start = Instant::now();
            schema.drop_all_tables().await?;
            operation("Dropping all tables", start.elapsed(), Status::Done)
        }
        // check migration table
        if !schema.repository_exists().await? {
            title(TitleKind::Info, "Preparing database.");
            schema.create_migration_table().await?;
        }
        batch = schema.get_next_batch_number().await?;
        schema.get_ran_migrations().await?
    } else {
        vec![]
    };
    let downs = schema.get_ran_migrations_gt(batch - (rollback + 1)).await?;

    if passive {
        title(TitleKind::Info, "Running model generator.");
    }else{
        title(TitleKind::Info, "Running migrations.");
    }

    for mig in migrations {
        let start = Instant::now();
        if rollback <= 0 {
            mig.up(&mut schema).await?;
            if !passive && !migration_list.contains(&mig.name().to_string()) {
                // run migration
                schema.execute_migration(mig.name(), &start.into()).await?;
                // add to table
                schema.add_migrated_table(mig.name(), batch).await?;

                migrated_count += 1;
            }
        } else {
            // println!("Rolling back {}, {:?}, {} , {}", mig.name(), downs, batch, batch - (rollback + 1));
            if downs.contains(&mig.name().to_string()) {
                mig.down(&mut schema).await?;
                migrated_count += 1;
                schema.rem_migrated_table(mig.name()).await?;
                operation(mig.name(), start.elapsed(), Status::Done);
            }
        }
    }

    if passive {
        for model_data in schema.to_struct().into_iter() {
            let start = Instant::now();
            let model_name = Str::ucfirst(&Str::singular(&model_data.table));
            match save_generated_model(model_name.clone(), Some(model_data)).await {
                Ok(_) => {
                    operation(&format!("Model generate: {}", &model_name), start.elapsed(), Status::Done);
                }
                Err(e) => {
                    operation(&format!("Model generate: {}", &model_name), start.elapsed(), Status::Failed);
                    eprintln!("Error: {:?}", e);
                }
            }
        }

    }

    if migrated_count == 0 && !passive {
        title(TitleKind::Info, "Noting to migrate");
    }
    Ok(())
}
