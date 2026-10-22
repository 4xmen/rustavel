// Bug 6: a database rule that can't be checked must fail, not pass
use macros::CheckMate;

#[derive(CheckMate)]
struct Signup { #[validating("unique:no_such_table,email")] email: String }

#[tokio::test]
async fn bug6_unique_on_unknown_table_is_an_error() {
    let r = Signup { email: "a@b.com".into() }.validate().await;
    assert!(r.is_err(), "unique passed although the table does not exist");
}