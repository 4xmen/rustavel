use rustavel_core::config::CONFIG;
use rustavel_core::db::schema::Schema;
use rustavel_core::helper::database::is_sqlite;
use rustavel_core::helper::number::random;
use std::time::Instant;

#[tokio::test]
async fn schema_create_and_drop_table_check_column() {
    let mut schema = Schema::new().await.unwrap();
    let table_foreign = format!("table_{}", random(111111, 999999));
    let table_name = format!("table_{}", random(111111, 999999));
    let a = Instant::now();

    schema.create(table_foreign.clone(), |table| {
        table.id();
        table
            .string("hafez", 10)
            .comment("is foreign protector sample :D");
    });
    schema
        .execute_migration("create table: hafez", &a.into())
        .await
        .unwrap();
    schema.create(table_name.clone(), |table| {
        table.id();
        table
            .string("title", 127)
            .index()
            .comment("test string")
            .nullable();
        table.big_integer("hafez_id").unsigned();
        table.string("email", 127).unique().comment("email unique");
        table
            .boolean("done")
            .default_bool(false)
            .comment("is task done");
        table
            .big_integer("parent_id")
            .nullable()
            .default_null()
            .unsigned();
        table.morph("todoable", "todoab").comment("morph todo");
        table.soft_delete();
        table.timestamps();
        table
            .foreign("parent_id")
            .on(&table_name)
            .reference("id")
            .cascade_on_delete();
        table
            .foreign("hafez_id")
            .on(&table_foreign)
            .reference("id")
            .cascade_on_delete();
    });

    schema
        .execute_migration("create table : test", &a.into())
        .await
        .unwrap();

    let actual = schema.get_foreign_keys(&table_name).await.unwrap();

    let expected = vec![
        format!("{}_hafez_id_foreign", &table_name),
        format!("{}_parent_id_foreign", &table_name),
    ];

    assert_eq!(actual, expected);

    schema.table(table_name.clone(), |table| {
        table.datetime("other");
        table.big_integer("another_ref").unsigned();
        table
            .datetime("another_index")
            .index()
            .nullable()
            .default_null();
        table
            .foreign("another_ref")
            .on(table_name.clone())
            .reference("id")
            .cascade_on_delete();
        table.rename_column("todoable_id", "td_id");
        table.rename_column("todoable_type", "td_type");
        table.drop_foreign(format!("{}_hafez_id_foreign", &table_name));
        table.drop_foreign(format!("{}_parent_id_foreign", &table_name));
    });
    schema
        .execute_migration("alter table: test", &a.into())
        .await
        .unwrap();

    assert!(schema.has_column(&table_name, "td_id").await.unwrap());
    assert!(schema.has_column(&table_name, "td_type").await.unwrap());

    let actual = schema.get_foreign_keys(&table_name).await.unwrap();

    if is_sqlite() {
        assert_eq!(actual, expected);
    } else {
        let expected = vec![format!("{}_another_ref_foreign", &table_name)];
        assert_eq!(actual, expected);
    }

    let tables = schema.get_tables().await.unwrap();
    assert!(tables.contains(&format!("{}{}", CONFIG.database.prefix, &table_name)),);
    assert!(
        schema
            .has_column(table_name.clone(), "other")
            .await
            .unwrap()
    );
    schema.drop_table(&table_name).await.unwrap();

    assert_eq!(true, true);
}
