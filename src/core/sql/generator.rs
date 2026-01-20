use std::fmt::Debug;

pub trait SqlGenerator: Debug {
    fn drop_table_if_exists(&self, table_name: &str) -> String;
}
