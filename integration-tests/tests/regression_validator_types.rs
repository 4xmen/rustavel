// Bugs 8 and 10: this file only compiles once both are fixed
use macros::CheckMate;

#[derive(CheckMate)]
struct Types {
    #[validating("max:10")] a: Option<i32>,
    #[validating("min:1|max:100")] b: f64,
    #[validating("min:1")] c: u64,
    #[validating("before:2030-01-01")] d: Option<time::Date>,
    #[validating("after:2020-01-01")] e: Option<time::PrimitiveDateTime>,
}

mod requests {
    #[derive(macros::CheckMate)]
    pub struct InOtherModule { #[validating("max:5")] pub s: String }
}

fn needs_trait<T: macros_core::CheckMateValidator>() {}

#[tokio::test]
async fn bug8_rules_compile_and_work_on_option_and_float() {
    let t = Types { a: Some(50), b: 0.5, c: 3, d: None, e: None };
    let errors = t.validate().await.unwrap_err();
    assert!(errors.errors.contains_key("a"), "Some(50) > max:10");
    assert!(errors.errors.contains_key("b"), "0.5 < min:1");
    assert!(!errors.errors.contains_key("c"));
}

#[tokio::test]
async fn bug10_validate_is_public_and_trait_is_implemented() {
    needs_trait::<requests::InOtherModule>();
    assert!(requests::InOtherModule { s: "abc".into() }.validate().await.is_ok());
}