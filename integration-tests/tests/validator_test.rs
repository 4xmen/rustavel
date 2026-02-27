use std::collections::HashMap;
use macros::CheckMate;
use rustavel_core::localization::digits::apply_normalized_string;
use rustavel_core::localization::numbers::apply_normalize_number;
use rustavel_core::facades::datetime::*;
use serde::{Deserialize, Serialize};
use time::{Date, OffsetDateTime, PrimitiveDateTime};

#[derive(Debug, Serialize, Deserialize, CheckMate)]
pub struct TestPayload {
    id: u64,
    #[validating("required|min:12|max:180")]
    pub title: String,
    #[serde(deserialize_with = "apply_normalized_string")]
    #[validating("required|alphanumeric")]
    pub code: String,
    #[serde(deserialize_with = "apply_normalize_number")]
    #[validating("required|min:18|max:900|size:2")]
    pub age: u16,
    #[serde(deserialize_with = "apply_normalize_number")]
    pub height: f64,

    #[serde(
        deserialize_with = "deserialize_date",
        serialize_with = "serialize_date"
    )]
    #[validating("required|date|before:2013-01-01")]
    pub dob: Date,
    #[serde(
        deserialize_with = "deserialize_datetime",
        serialize_with = "serialize_datetime"
    )]
    #[validating("required|datetime|after:2020-01-01")]
    pub published: PrimitiveDateTime,

    #[validating("nullable|date|after:2020-01-01|before:2000-03-12")]
    pub omg: Option<String>,

    #[validating("nullable|min:10|confirmed:pass_confirm")]
    pub pass: Option<String>,
    pub pass_confirm: Option<String>,

    #[validating("required|not_in:admin,user,guest")]
    role: String,
    #[validating("required|array")]
    items: Vec<String>,
    //
    // #[validating("required|email|exists:users,email")]
    // email: String,
    //
    // #[validating("required|email|unique:users,email,id")]
    // email2: String,
}

// helper for check error
fn assert_has_error(
    map: &HashMap<String, Vec<String>>,
    field: &str,
    expected_substring: &str,
) {
    let field_errors = map.get(field)
        .unwrap_or_else(|| panic!("Field '{}' should have errors", field));

    assert!(
        field_errors.iter().any(|msg| msg.contains(expected_substring)),
        "Expected '{}' inside errors of '{}'. Got: {:?}",
        expected_substring,
        field,
        field_errors
    );
}

#[tokio::test]
async fn test_validator_correct() {
    let date = parse_ymd("2000-10-01").unwrap();

    let payload = TestPayload {
        id: 1,
        title: "this a title".to_string(),
        code: "code0099".to_string(),
        age: 25,
        height: 15.0,
        dob: date,
        published: now_primitive(),
        pass: Some("pass_confirm".to_string()),
        pass_confirm: Some("pass_confirm".to_string()),
        role: "not-admin".to_string(),
        items: vec!["item1".to_string(), "item2".to_string()],
        omg: None,
    };

    let data = payload.validate().await;
    assert_eq!(data.is_ok(), true);
}
#[tokio::test]
async fn test_validator_not_correct() {
    let now = OffsetDateTime::now_utc();

    let payload = TestPayload {
        id: 1,
        title: "short".to_string(),
        code: "code0099 راستاول".to_string(),
        age: 10,
        height: 15.0,
        dob: now.date(),
        published: now_primitive(),
        pass: Some("shortpass".to_string()),
        pass_confirm: Some("pass_not_confirm".to_string()),
        role: "admin".to_string(),
        items: vec!["item1".to_string(), "item2".to_string()],
        omg: Some( "2019-12-29".to_string() ),
    };

    let result = payload.validate().await;
    let err = result.expect_err("Expected validation to fail");
    let map = &err.errors;

    // --------- check field count ----------
    assert_eq!(map.len(), 7);

    // --------- age ----------
    assert_has_error(map, "age", "greater than or equal to 18");

    // --------- title ----------
    assert_has_error(map, "title", "at least `12`");

    // --------- pass ----------
    {
        let pass_errors = map.get("pass").expect("pass should exist");
        assert_eq!(pass_errors.len(), 2);

        assert_has_error(map, "pass", "at least `10`");
        assert_has_error(map, "pass", "not confirmed");
    }

    // --------- role ----------
    assert_has_error(map, "role", "must not be one of the allowed options");

    // --------- code ----------
    assert_has_error(map, "code", "Alphanumeric");

    // --------- omg ----------
    {
        let omg_errors = map.get("omg").expect("omg should exist");
        assert_eq!(omg_errors.len(), 2);

        assert_has_error(map, "omg", "must be later than");
        assert_has_error(map, "omg", "must be earlier than");
    }

    // --------- dob ----------
    assert_has_error(map, "dob", "must be earlier than 2013-01-01");

}