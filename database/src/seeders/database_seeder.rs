use anyhow::Result;
use crate::seeders;

use crate::seeders::todo_seeder::TodoSeeder;


pub struct DatabaseSeeder;

impl DatabaseSeeder {
    /**
     * Seed the application's database.
     */
    pub async fn run() -> Result<()> {
        let _ = seeders!(TodoSeeder);
        Ok(())
    }

}
