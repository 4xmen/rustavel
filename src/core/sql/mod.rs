pub mod generator;
pub mod sqlite;
pub mod mysql;
use crate::config::database::DatabaseEngine;
use crate::core::sql::generator::{SqlGenerator};
use crate::core::sql::{mysql::MySqlGenerator, sqlite::SqliteGenerator};

pub fn get_generator(engine: &DatabaseEngine) -> Box<dyn SqlGenerator> {
    match engine {
        DatabaseEngine::Mysql => Box::new(MySqlGenerator),
        DatabaseEngine::Sqlite => Box::new(SqliteGenerator),
    }
}
