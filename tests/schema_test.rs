use rustavel::core::schema::*;

#[test]
fn simple_schema_test() {
    let s = Schema::new();
    let sql = s.drop_if_exists("users");
}
