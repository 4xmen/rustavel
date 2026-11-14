// Bugs 5, 18 and 20. Run with DB_CONNECTION=Sqlite (see the guide).
use macros::CheckMate;
use rustavel_core::config::{CONFIG, database::DatabaseEngine};
use rustavel_core::db::schema::Schema;
use rustavel_core::helper::database::is_sqlite;
use std::time::Instant;

fn sqlite() -> bool {
    let ok = is_sqlite();
    if !ok {
        eprintln!("skipped: run with DB_CONNECTION=Sqlite");
    }
    ok
}

#[derive(CheckMate)]
struct Signup {
    #[validating("unique:reg_users,email")]
    email: String,
}
#[derive(CheckMate)]
struct Login {
    #[validating("exists:reg_users,email")]
    email: String,
}

#[tokio::test]
async fn sqlite_suite() {
    if !sqlite() {
        return;
    }

    let mut s = Schema::new().await.unwrap();

    let table_name = format!("{}reg_users", &CONFIG.database.prefix);

    // check default remove last tested tables
    s.drop_table_raw_name_if_exists("app_reg_users").await;
    s.drop_table_if_exists("reg_users").await;

    s.create("reg_users", |t| {
        t.id();
        t.string("email", 100);
    });
    s.execute_migration("reg_users", &Instant::now().into())
        .await
        .unwrap();

    s.client
        .execute(&format!(
            "INSERT INTO {} (email) VALUES ('a@b.com');",
            &table_name
        ))
        .await
        .unwrap();

    let mut failures: Vec<String> = vec![];

    // bug 5
    if (Signup {
        email: "a@b.com".into(),
    })
    .validate()
    .await
    .is_ok()
    {
        failures.push("bug 5: unique missed a duplicate".into());
    }
    if (Login {
        email: "a@b.com".into(),
    })
    .validate()
    .await
    .is_err()
    {
        failures.push("bug 5: exists missed an existing row".into());
        panic!("{:?}", failures);
    }

    // bug 20
    s.table("reg_users", |t| {
        t.drop_column("email");
    });
    s.execute_migration("drop email", &Instant::now().into())
        .await
        .unwrap();
    if s.has_column("reg_users", "email").await.unwrap() {
        failures.push("bug 20: drop_column did nothing".into());
    }

    // bug 18 (renames every table in this SQLite file)
    s.rename_prefix("app_").await.unwrap();
    let tables = s.get_tables().await.unwrap();
    if !tables.contains(&"app_reg_users".to_string()) {
        failures.push(format!("bug 18: tables are now {:?}", tables));
    }
    s.drop_table_raw_name_if_exists("app_reg_users").await;
    assert!(failures.is_empty(), "{:#?}", failures);
}
