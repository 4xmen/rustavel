#[allow(unused_imports)]
use std::time::Instant;
use anyhow::Result;
use crate::seeders;
#[allow(unused_imports)]
use rustavel_core::facades::terminal_ui::{operation, Status,title, TitleKind};


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
