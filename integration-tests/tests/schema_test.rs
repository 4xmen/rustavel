use std::time::Instant;
use rustavel_core::db::schema::Schema;
use rustavel_core::facades::number::random;
#[tokio::test]
async fn schema_create_and_drop_table_check_column() {
    let mut schema = Schema::new().await.unwrap();
    let table_name = format!("table_{}", random(111111,999999));
    schema.create(table_name.clone(), |table| {
        table.id();
        table.string("title", 127).index().comment("test string").nullable();
        table.string("email",127).unique().comment("email unique");
        table.boolean("done").default_bool(false).comment("is task done");
        table.big_integer("parent_id").nullable().default_null().unsigned();
        table.soft_delete();
        table.timestamps();
        table.foreign("parent_id").on(table_name.clone()).reference("id").cascade_on_delete();
    });

    let a = Instant::now();
    schema.execute_migration("create table",&a.into()).await.unwrap();

    schema.table(table_name.clone(), |table| {
        table.datetime("other");
        table.big_integer("another_ref").unsigned();
        table.datetime("another_index").nullable().default_null();
        table.foreign("another_ref").on(table_name.clone()).reference("id").cascade_on_delete();
    });
    schema.execute_migration("alter table",&a.into()).await.unwrap();

    let tables = schema.get_tables().await.unwrap();
    assert_eq!(tables.contains(&table_name),true);
    assert_eq!(schema.has_column(table_name.clone(),"other").await.unwrap(),true);
    schema.drop_table(&table_name).await.unwrap();

    assert_eq!(true,true);


}