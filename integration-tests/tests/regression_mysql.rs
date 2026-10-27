// Bugs 14, 15, 16, 17 (SQL that MySQL receives) and 19 (needs MySQL)
use rustavel_core::config::{database::DatabaseEngine, CONFIG};
use rustavel_core::db::schema::Schema;
use rustavel_core::db::table::{Column, ColumnDataType, ColumnOption, DefaultValue, ForeignKey, TableAction};
use rustavel_core::sql::generator::SqlGenerator;
use rustavel_core::sql::mysql::MySqlGenerator;
use std::time::Instant;

fn col(name: &str, data_type: ColumnDataType) -> Column {
    Column { name: name.into(), data_type, option: ColumnOption::Length(50), ..Default::default() }
}

#[test]
fn bug14_cascade_on_delete_means_on_delete() {
    let key = ForeignKey { column_name: "parent_id".into(), foreign_table: "parent".into(),
        referenced_column: "id".into(), on_delete: true, on_update: false };
    let sql = MySqlGenerator.foreign_key(&key, "child", &TableAction::Create);
    assert!(sql.contains("ON DELETE CASCADE") && !sql.contains("ON UPDATE"), "got: {sql}");
}

#[test]
fn bug15_json_check_uses_its_own_column() {
    let (_, footer, _, _) = MySqlGenerator.column(&col("meta", ColumnDataType::DTJson), "t", &TableAction::Create);
    assert!(footer.contains("json_valid(`meta`)"), "got: {footer}");
}

#[test]
fn bug16_quotes_are_escaped() {
    let mut c = col("note", ColumnDataType::DTString);
    c.default = DefaultValue::String("It's".into());
    c.comment = "user's name".into();
    let (sql, _, _, _) = MySqlGenerator.column(&c, "t", &TableAction::Create);
    assert!(sql.contains("'It''s'") && sql.contains("'user''s name'"), "got: {sql}");
}

#[test]
fn bug17_alter_unique_gets_add() {
    let mut c = col("email", ColumnDataType::DTString);
    c.unique = true;
    let (_, footer, _, _) = MySqlGenerator.column(&c, "t", &TableAction::Alter);
    assert!(footer.starts_with("ADD UNIQUE INDEX"), "got: {footer}");
}

#[tokio::test]
async fn bug19_has_index_needs_one_index_on_all_columns() {
    if CONFIG.database.connection != DatabaseEngine::Mysql { eprintln!("skipped: needs MySQL"); return; }
    let mut s = Schema::new().await.unwrap();
    s.drop_table_if_exists("reg_idx").await;
    s.create("reg_idx", |t| { t.id(); t.string("a", 20).index(); t.string("b", 20).index(); });
    s.execute_migration("reg_idx", &Instant::now().into()).await.unwrap();
    let found = s.has_index("reg_idx", vec!["a", "b"]).await.unwrap();
    s.drop_table("reg_idx").await.unwrap();
    assert!(!found, "a and b have separate indexes, not one index on both");
}