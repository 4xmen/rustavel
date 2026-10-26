// Bugs 3, 4, 7 and 9: validator rules that run but give the wrong answer
use macros::CheckMate;

#[derive(CheckMate)]
struct AfterDate { #[validating("after:2024-01-01")] d: String }
#[derive(CheckMate)]
struct BeforeDate { #[validating("before:2030-01-01")] d: String }
#[derive(CheckMate)]
struct AfterDateTime { #[validating("after:2024-01-01 10:00:00")] d: time::PrimitiveDateTime }
#[derive(CheckMate)]
struct Confirm {
    #[validating("confirmed:pass_confirm")] pass: Option<String>,
    pass_confirm: Option<String>,
}
#[derive(CheckMate)]
struct MaxFive { #[validating("max:5")] s: String }
#[derive(CheckMate)]
struct SizeFour { #[validating("size:4")] s: String }

#[tokio::test]
async fn bug3_after_and_before_accept_a_valid_date() {
    assert!(AfterDate { d: "2025-05-05".into() }.validate().await.is_ok(), "2025-05-05 is after 2024-01-01");
    assert!(AfterDate { d: "2023-05-05".into() }.validate().await.is_err(), "2023-05-05 is not after 2024-01-01");
    assert!(BeforeDate { d: "2025-05-05".into() }.validate().await.is_ok(), "2025-05-05 is before 2030-01-01");
}

#[tokio::test]
async fn bug4_confirmed_fails_when_confirmation_is_missing() {
    let r = Confirm { pass: Some("secret123".into()), pass_confirm: None }.validate().await;
    assert!(r.is_err(), "password without confirmation must fail");
    let r = Confirm { pass: Some("a".into()), pass_confirm: Some("a".into()) }.validate().await;
    assert!(r.is_ok());
}

#[tokio::test]
async fn bug7_datetime_target_with_time_works() {
    let d = time::macros::datetime!(2025-05-05 00:00:00);
    assert!(AfterDateTime { d }.validate().await.is_ok());
}

#[tokio::test]
async fn bug9_length_rules_count_characters() {
    assert!(MaxFive { s: "سلام".into() }.validate().await.is_ok(), "4 characters, 8 bytes");
    assert!(SizeFour { s: "سلام".into() }.validate().await.is_ok());
}

// #[derive(CheckMate)]
// struct BadFormat {
//     #[validating("after:2024/01/01")]
//     d: String,
// }