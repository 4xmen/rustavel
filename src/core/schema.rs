use crate::core::table::{Column, Table, TableAction};

#[derive(Debug)]
pub struct Schema<'a> {
    table_prefix: &'a str,
}

impl Schema<'_> {
    pub fn new() -> Self {
        Self { table_prefix: "" }
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
    pub fn drop_if_exists(table_name: &str) {}
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
}
