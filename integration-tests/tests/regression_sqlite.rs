// Bugs 5, 18 and 20. Run with DB_CONNECTION=Sqlite (see the guide).
use macros::CheckMate;
use rustavel_core::config::{database::DatabaseEngine, CONFIG};
use rustavel_core::db::schema::Schema;
use std::time::Instant;

fn sqlite() -> bool {
    let ok = CONFIG.database.connection == DatabaseEngine::Sqlite;
    if !ok { eprintln!("skipped: run with DB_CONNECTION=Sqlite"); }
    ok
}

#[derive(CheckMate)]
struct Signup { #[validating("unique:reg_users,email")] email: String }
#[derive(CheckMate)]
struct Login { #[validating("exists:reg_users,email")] email: String }

#[tokio::test]
async fn sqlite_suite() {
    if !sqlite() { return; }
    let mut s = Schema::new().await.unwrap();
    s.drop_table_if_exists("reg_users").await;
    s.create("reg_users", |t| { t.id(); t.string("email", 100); });
    s.execute_migration("reg_users", &Instant::now().into()).await.unwrap();
    std::process::Command::new("sqlite3").arg(&CONFIG.database.database)
        .arg("INSERT INTO reg_users (email) VALUES ('a@b.com');").status().unwrap();

    let mut failures: Vec<String> = vec![];

    // bug 5
    if (Signup { email: "a@b.com".into() }).validate().await.is_ok() {
        failures.push("bug 5: unique missed a duplicate".into());
    }
    if (Login { email: "a@b.com".into() }).validate().await.is_err() {
        failures.push("bug 5: exists missed an existing row".into());
    }

    // bug 20
    s.table("reg_users", |t| { t.drop_column("email"); });
    s.execute_migration("drop email", &Instant::now().into()).await.unwrap();
    if s.has_column("reg_users", "email").await.unwrap() {
        failures.push("bug 20: drop_column did nothing".into());
    }

    // bug 18 (renames every table in this SQLite file)
    s.rename_prefix("app_").await.unwrap();
    let tables = s.get_tables().await.unwrap();
    if !tables.contains(&"app_reg_users".to_string()) {
        failures.push(format!("bug 18: tables are now {:?}", tables));
    }

    assert!(failures.is_empty(), "{:#?}", failures);
}