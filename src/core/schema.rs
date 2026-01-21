use crate::config::CONFIG;
use crate::config::database::DatabaseEngine;
use crate::core::sql::database_client::DatabaseClient;
use crate::core::sql::database_client::MySqlClient;
use crate::core::sql::database_client::SqliteClient;
use crate::core::sql::generator::SqlGenerator;
use crate::core::sql::mysql::MySqlGenerator;
use crate::core::sql::sqlite::SqliteGenerator;
use crate::core::table::{Column, Table, TableAction};
use sqlx::{MySqlPool, SqlitePool};

#[allow(dead_code)]
#[derive(Debug)]
pub struct Schema  {
    prefix: String,
    generator: Box<dyn SqlGenerator + Send + Sync>,
    client: Box<dyn DatabaseClient + Send + Sync>,
}

impl Schema {
    pub async fn new() -> Self {
        let (generator, client): (Box<dyn SqlGenerator>, Box<dyn DatabaseClient>) =
            match CONFIG.database.connection {
                DatabaseEngine::Mysql => {
                    let con_string = format!(
                        "mysql://{}:{}@{}:{}/{}",
                        &CONFIG.database.username,
                        &CONFIG.database.password,
                        &CONFIG.database.host,
                        &CONFIG.database.port,
                        &CONFIG.database.database,
                    );
                    let pool = MySqlPool::connect(&con_string).await.unwrap();
                    (Box::new(MySqlGenerator), Box::new(MySqlClient { pool }))
                }
                DatabaseEngine::Sqlite => {
                    let pool = SqlitePool::connect(&CONFIG.database.database)
                        .await
                        .unwrap();
                    (Box::new(SqliteGenerator), Box::new(SqliteClient { pool }))
                }
            };

        Self {
            prefix: CONFIG.database.prefix.clone(),
            generator,
            client,
        }
    }
    pub fn get_tables() -> Vec<&'static str> {
        vec![]
    }
    pub fn get_views() -> Vec<&'static str> {
        vec![]
    }
    pub fn get_current_schema_name() -> &'static str {
        ""
    }
    pub fn get_column_listing() -> Vec<&'static str> {
        vec![]
    }
    pub fn get_columns(table_name: &str) -> Vec<Column> {
        vec![]
    }
    pub fn get_schemas() -> Vec<&'static str> {
        vec![]
    }
    pub fn get_foreign_keys(table_name: &str) -> Vec<&str> {
        vec![]
    }

    pub fn drop(table_name: &str) {}
    pub async fn drop_table_if_exists(&self, table: &str) {
        let sql = self
            .generator
            .drop_table_if_exists(&self.fix_table_name(table));
        self.client.execute(&sql).await.unwrap();
    }

    pub fn drop_all_tables() {}
    pub fn drop_all_views() {}

    pub fn has_column(table_name: &str, column_name: &str) -> bool {
        false
    }
    pub fn has_table(table_name: &str, column_name: &str) -> bool {
        false
    }
    pub fn has_view(table_name: &str, column_name: &str) -> bool {
        false
    }
    pub fn has_index(table_name: &str, columns_name: Vec<&str>) -> bool {
        false
    }
    pub fn create_database(db_name: &str) -> bool {
        false
    }
    pub fn drop_database_if_exists(db_name: &str) -> bool {
        false
    }
    pub fn disable_foreign_key_constraints(db_name: &str) -> bool {
        false
    }
    pub fn enable_foreign_key_constraints(db_name: &str) -> bool {
        false
    }

    pub fn rename(old_table_name: &str, new_table_name: &str) {}

    pub fn rename_prefix(old_prefix: &str, new_prefix: &str) {}
    pub fn add_prefix(prefix: &str) {}

    pub fn create<F>(table_name: &str, f: F)
    where
        F: FnOnce(&mut Table),
    {
        let mut table = Table::new(table_name);
        table.action = TableAction::Create;
        f(&mut table);
    }

    pub fn table<F>(table_name: &str, f: F)
    where
        F: FnOnce(&mut Table),
    {
        let mut table = Table::new(table_name);
        table.action = TableAction::Alter;
        f(&mut table);
    }

    fn fix_table_name(&self, table_name: &str) -> String {
        format!("{}{}", self.prefix, table_name)
    }
}
