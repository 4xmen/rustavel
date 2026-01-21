use std::fmt::Debug;

pub trait SqlGenerator: Debug  + Sync + Send {
    fn drop_table_if_exists(&self, table_name: &str) -> String;
}
