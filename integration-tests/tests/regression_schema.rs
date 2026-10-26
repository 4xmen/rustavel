// Bugs 11, 22 and 25
use rustavel_core::db::schema::Schema;
use rustavel_core::db::table::Table;
use std::time::Instant;

#[tokio::test]
async fn bug11_every_create_in_one_migration_runs() {
    let mut s = Schema::new().await.unwrap();
    s.drop_table_if_exists("reg_one").await;
    s.drop_table_if_exists("reg_two").await;
    s.create("reg_one", |t| { t.id(); });
    s.create("reg_two", |t| { t.id(); });
    s.execute_migration("two creates", &Instant::now().into()).await.unwrap();
    let one = s.has_table("reg_one").await.unwrap();
    s.drop_table_if_exists("reg_one").await;
    s.drop_table_if_exists("reg_two").await;
    assert!(one, "only the last create() ran");
}

#[test]
fn bug22_model_types_match_columns() {
    let mut t = Table::new("posts");
    t.timestamps();
    t.morph("owner", "owner_index");
    t.datetime("seen_at").unsigned();
    let model = t.to_struct();
    assert!(model.contains("pub created_at: Option<time::PrimitiveDateTime>"), "{model}");
    assert!(model.contains("pub owner_type:") && model.contains("pub owner_id:"), "{model}");
    assert!(model.contains("time::PrimitiveDateTime") && !model.contains("tume::"), "{model}");
}

#[tokio::test]
async fn bug25_primary_key_is_per_table() {
    let mut s = Schema::new().await.unwrap();
    for i in 0..6 { s.create(format!("withid{i}"), |t| { t.id(); }); }
    s.create("noid", |t| { t.string("code", 10); });
    let pk = s.to_struct().into_iter().find(|m| m.table == "noid").unwrap().primary_key;
    assert_eq!(pk, "", "a table without id got primary key \"id\"");
}