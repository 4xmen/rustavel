use rustavel_core::localization::numbers::apply_normalize_number;
use rustavel_core::facades::datetime::*;
use serde::{Deserialize, Serialize};
use time::{Date, PrimitiveDateTime};

#[derive(Debug, Serialize, Deserialize)]
pub struct SerializePayload {
    #[serde(deserialize_with = "apply_normalize_number")]
    pub age: u16,
    #[serde(deserialize_with = "apply_normalize_number")]
    pub height: f64,

    #[serde(
        deserialize_with = "deserialize_date",
        serialize_with = "serialize_date"
    )]
    pub dob: Date,
    #[serde(
        deserialize_with = "deserialize_datetime",
        serialize_with = "serialize_datetime"
    )]
    pub published: PrimitiveDateTime,
}

#[test]
fn should_normalize_persian_numbers() {
    let json = r#"
    {
        "age": "۳9",
        "height": "۱۷۵/۵",
        "dob": "۲۰۱۰-۰۵-۱۲",
        "published": "2020-03-01 10:30:22"
    }
    "#;

    let payload: SerializePayload =
        serde_json::from_str(json).expect("Deserialization should succeed");

    println!("{:?}", payload);
    assert_eq!(payload.age, 39);
    assert_eq!(payload.height, 175.5);

    // تاریخ‌ها رو هم چک می‌کنیم
    assert_eq!(payload.dob.year(), 2010);
    assert_eq!(payload.dob.month() as u8, 5);
    assert_eq!(payload.dob.day(), 12);

    assert_eq!(payload.published.year(), 2020);
    assert_eq!(payload.published.month() as u8, 3);
    assert_eq!(payload.published.day(), 1);
    assert_eq!(payload.published.hour(), 10);
}

#[derive(Debug, Deserialize)]
struct NumberOnly {
    #[serde(deserialize_with = "apply_normalize_number")]
    value: u16,
}

#[test]
fn should_not_parse()
{
    let json = r#"{ "value": "۴۲ac" }"#;

    let result: Result<NumberOnly, _> = serde_json::from_str(json);
    assert!(result.is_err());
}