use super::generator::SqlGenerator;

#[derive(Debug)]
pub struct MySqlGenerator;

impl SqlGenerator for MySqlGenerator {
    fn drop_table_if_exists(&self, table_name: &str) -> String {
        format!("DROP TABLE IF EXISTS `{}`;", table_name)
    }

}
