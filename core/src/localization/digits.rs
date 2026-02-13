use serde::{Deserializer, Deserialize};

pub fn apply_normalized_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(normalize_digits(&s))
}
pub fn normalize_digits(input: &str) -> String {
    input.chars().map(|c| match c {
        '۰' | '٠' => '0',
        '۱' | '١' => '1',
        '۲' | '٢' => '2',
        '۳' | '٣' => '3',
        '۴' | '٤' => '4',
        '۵' | '٥' => '5',
        '۶' | '٦' => '6',
        '۷' | '٧' => '7',
        '۸' | '٨' => '8',
        '۹' | '٩' => '9',
        _ => c,
    }).collect()
}