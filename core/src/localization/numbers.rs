use crate::localization::digits::normalize_digits;
use serde::{Deserializer, Deserialize};

pub fn apply_normalize_number<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    let s = String::deserialize(deserializer)?;
    let normalized = normalize_digits(&s);
    normalized.parse::<T>().map_err(serde::de::Error::custom)
}