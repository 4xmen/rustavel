use crate::config::CONFIG;
use crate::config::database::{DatabaseEngine};
use crate::core::table::{Column, Table, TableAction};
use crate::core::sql::generator::{SqlGenerator};
use crate::core::sql::mysql::MySqlGenerator;
use crate::core::sql::sqlite::SqliteGenerator;

#[derive(Debug)]
pub struct Schema {
    prefix:  String,
    generator: Box<dyn SqlGenerator>,
}

impl Schema {
    pub fn new() -> Self {
        let generator: Box<dyn SqlGenerator> = match CONFIG.database.connection {
            DatabaseEngine::Mysql => Box::new(MySqlGenerator),
            DatabaseEngine::Sqlite => Box::new(SqliteGenerator),
        };

        Self {
            generator,
            prefix: CONFIG.database.prefix.clone(),
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
    pub fn drop_if_exists(&self,table_name: &str) {
        let sql = self.generator.drop_table_if_exists(&self.fix_table_name(table_name));
        println!("{}", sql);
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
        format!("{}{}",self.prefix, table_name)
    }
}
