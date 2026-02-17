use crate::db::table::{Column, ForeignKey, TableAction};
use std::fmt::Debug;

pub trait SqlGenerator: Debug + Sync + Send {
    fn drop_table_if_exists(&self, table_name: &str) -> String;
    fn get_tables(&self) -> String;
    fn get_views(&self) -> String;
    fn get_column_listing(&self, table_name: &str) -> String;
    fn get_foreign_keys(&self, table_name: &str) -> String;
    fn drop_table(&self, table_name: &str) -> String;
    fn drop_view(&self, view_name: &str) -> String;
    // fn drop_all_tables(&self) -> String;
    // fn drop_all_views(&self) -> String;
    fn has_column(&self, table_name: &str, column_name: &str) -> String;
    fn has_table(&self, table_name: &str) -> String;
    fn has_view(&self, table_name: &str) -> String;
    fn has_index(&self, table_name: &str, columns_name: Vec<&str>) -> String;
    fn create_database(&self, db_name: &str) -> String;
    fn drop_database_if_exists(&self, db_name: &str) -> String;
    fn disable_foreign_key_constraints(&self) -> String;
    fn enable_foreign_key_constraints(&self) -> String;
    fn rename(&self, old_table_name: &str, new_table_name: &str) -> String;
    fn column(
        &self,
        column: &Column,
        table_name: &str,
        action: &TableAction,
    ) -> (String, String, String);
    fn foreign_key(&self, key: &ForeignKey, table_name: &str, action: &TableAction) -> String;
    fn drop_column(&self, column_name: &str) -> String;

    fn table_sql(
        &self,
        table_name: &str,
        body_sql: &str,
        post_sql: &str,
        action: &TableAction,
    ) -> String;

    fn get_ran(&self) -> String;
    fn get_ran_gt(&self) -> String;

    fn get_next_batch_number(&self) -> String;

    fn add_migrated_table(&self) -> String;
    fn rem_migrated_table(&self) -> String;
    
    fn record_exists(&self,table: &str,column: &str) -> String;

}
