use super::generator::SqlGenerator;

#[derive(Debug)]
pub struct SqliteGenerator;

impl SqlGenerator for SqliteGenerator {
    fn drop_table_if_exists(&self, table_name: &str) -> String {
        format!("DROP TABLE IF EXISTS \"{}\";", table_name)
    }
}
