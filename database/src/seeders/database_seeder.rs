use crate::seeders::todo_seeder::TodoSeeder;
use anyhow::Result;

pub struct DatabaseSeeder;

impl DatabaseSeeder {
    /**
     * Seed the application's database.
     */
    pub async fn run() -> Result<()> {
        TodoSeeder::run().await?;
        Ok(())
    }
}