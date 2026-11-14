use rustavel_core::config::CONFIG;
use rustavel_core::db::schema::Schema;
use rustavel_core::helper::database::is_sqlite;
use rustavel_core::helper::number::random;
use std::fmt::Debug;
use std::time::Instant;

fn assert_eq_at<T: Debug + PartialEq>(stage: &str, actual: T, expected: T) -> Result<(), String> {
    if actual != expected {
        return Err(format!(
            "[{stage}] Assertion failed.\nExpected: {expected:#?}\nActual: {actual:#?}"
        ));
    }

    Ok(())
}

async fn create_fixture_tables(
    schema: &mut Schema,
    foreign_table: &str,
    table_name: &str,
) -> Result<(), String> {
    let started_at = Instant::now();
    // Create the referenced table.
    schema.create(foreign_table.to_string(), |table| {
        table.id();
        table
            .string("hafez", 10)
            .comment("is foreign protector sample :D");
    });

    schema
        .execute_migration("create table: hafez", &started_at.into())
        .await
        .map_err(|e| format!("[CREATE REFERENCED TABLE] {e:?}"))?;

    // Create the main table and its initial foreign keys.
    schema.create(table_name.to_string(), |table| {
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
            .on(table_name)
            .reference("id")
            .cascade_on_delete();

        table
            .foreign("hafez_id")
            .on(foreign_table)
            .reference("id")
            .cascade_on_delete();
    });

    schema
        .execute_migration("create table: test", &started_at.into())
        .await
        .map_err(|e| format!("[CREATE MAIN TABLE] {e:?}"))?;

    Ok(())
}

async fn assert_initial_foreign_keys(schema: &Schema, table_name: &str) -> Result<(), String> {
    let actual = schema
        .get_foreign_keys(table_name)
        .await
        .map_err(|e| format!("[READ INITIAL FOREIGN KEYS] {e:?}"))?;
    let prefix = &CONFIG.database.prefix;
    let expected = vec![
        format!("{prefix}{table_name}_hafez_id_foreign"),
        format!("{prefix}{table_name}_parent_id_foreign"),
    ];

    assert_eq_at("INITIAL FOREIGN KEYS", actual, expected)
}

async fn alter_fixture_table(schema: &mut Schema, table_name: &str) -> Result<(), String> {
    let started_at = Instant::now();
    let full_table_name = &format!("{}{}", &CONFIG.database.prefix, table_name);
    // Add columns, rename existing columns and modify foreign keys.
    schema.table(table_name.to_string(), |table| {
        table.datetime("other");
        table.big_integer("another_ref").unsigned();

        table
            .datetime("another_index")
            .index()
            .nullable()
            .default_null();

        table
            .foreign("another_ref")
            .on(table_name.to_string())
            .reference("id")
            .cascade_on_delete();

        table.rename_column("todoable_id", "td_id");
        table.rename_column("todoable_type", "td_type");

        table.drop_foreign(format!("{full_table_name}_hafez_id_foreign"));
        table.drop_foreign(format!("{full_table_name}_parent_id_foreign"));
    });

    schema
        .execute_migration("alter table: test", &started_at.into())
        .await
        .map_err(|e| format!("[ALTER MAIN TABLE] {e:?}"))?;

    Ok(())
}

async fn assert_altered_table(schema: &Schema, table_name: &str) -> Result<(), String> {
    // Verify renamed columns.
    for column in ["td_id", "td_type"] {
        let exists = schema
            .has_column(table_name, column)
            .await
            .map_err(|e| format!("[CHECK COLUMN: {column}] {e:?}"))?;

        if !exists {
            return Err(format!(
                "[RENAMED COLUMNS] Expected column `{column}` to exist"
            ));
        }
    }

    // Verify the remaining foreign keys.
    let actual = schema
        .get_foreign_keys(table_name)
        .await
        .map_err(|e| format!("[READ ALTERED FOREIGN KEYS] {e:?}"))?;
    let prefix = &CONFIG.database.prefix;
    let expected = if is_sqlite() {
        vec![
            format!("{prefix}{table_name}_hafez_id_foreign"),
            format!("{prefix}{table_name}_parent_id_foreign"),
        ]
    } else {
        vec![format!("{prefix}{table_name}_another_ref_foreign")]
    };

    assert_eq_at("ALTERED FOREIGN KEYS", actual, expected)?;

    // Verify the newly added column.
    let exists = schema
        .has_column(table_name, "other")
        .await
        .map_err(|e| format!("[CHECK ADDED COLUMN] {e:?}"))?;

    if !exists {
        return Err("[ADDED COLUMN] Expected column `other` to exist".to_string());
    }

    // Verify the table is registered in the database.
    let tables = schema
        .get_tables()
        .await
        .map_err(|e| format!("[READ TABLE LIST] {e:?}"))?;

    let expected_table = format!("{}{}", CONFIG.database.prefix, table_name);

    if !tables.contains(&expected_table) {
        return Err(format!(
            "[TABLE REGISTRATION] Expected `{expected_table}` in the table list"
        ));
    }

    Ok(())
}

async fn cleanup_tables(schema: &mut Schema, table_name: &str, foreign_table: &str) -> Vec<String> {
    let mut errors = Vec::new();

    // Drop the referencing table first to avoid foreign key dependencies.
    for table in [table_name, foreign_table] {
        if let Err(error) = schema.drop_table(table).await {
            errors.push(format!("Failed to drop `{table}`: {error:?}"));
        }
    }

    errors
}

#[tokio::test]
async fn schema_create_and_drop_table_check_column() {
    let mut schema = Schema::new().await.expect("Failed to initialize schema");

    let foreign_table = format!("table_{}", random(111111, 999999));
    let table_name = format!("table_{}", random(111111, 999999));

    // Run all test stages and preserve the first failure.
    let test_result: Result<(), String> = async {
        create_fixture_tables(&mut schema, &foreign_table, &table_name).await?;

        assert_initial_foreign_keys(&schema, &table_name).await?;

        alter_fixture_table(&mut schema, &table_name).await?;

        assert_altered_table(&schema, &table_name).await?;

        Ok(())
    }
    .await;

    // Always attempt cleanup, even if a test stage returns an error.
    let cleanup_errors = cleanup_tables(&mut schema, &table_name, &foreign_table).await;

    if let Err(error) = test_result {
        panic!("Schema integration test failed:\n{error}\nCleanup errors: {cleanup_errors:#?}");
    }

    assert!(
        cleanup_errors.is_empty(),
        "Schema test cleanup failed:\n{cleanup_errors:#?}"
    );
}
