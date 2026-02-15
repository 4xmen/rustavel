use crate::localization::digits::normalize_digits;
use serde::{ Deserializer, de::{self, Visitor}};
use std::fmt;
use std::str::FromStr;

/// Deserializer that accepts:
/// - string (possibly containing eastern arabic / persian digits)
/// - integer number (i64, u64, ... that fits in target type)
/// - float number (if target is f32/f64)
pub fn apply_normalize_number<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: fmt::Display,
    T: de::Deserialize<'de>,
{
    struct NumberVisitor<T>(std::marker::PhantomData<T>);

    impl<'de, T> Visitor<'de> for NumberVisitor<T>
    where
        T: FromStr,
        T::Err: fmt::Display,
        T: de::Deserialize<'de>,
    {
        type Value = T;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string with digits or a number (integer/float)")
        }

        // parse normal int
        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            T::deserialize(de::value::I64Deserializer::<E>::new(v))
                .map_err(de::Error::custom)
        }
        // parse unsigned int
        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            T::deserialize(de::value::U64Deserializer::<E>::new(v))
                .map_err(de::Error::custom)
        }
        // parse normal float
        fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            T::deserialize(serde::de::value::F64Deserializer::<E>::new(v))
                .map_err(de::Error::custom)
        }

       // normalize persian/arabic digits
        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            let normalized = normalize_digits(v);
            normalized
                .parse::<T>()
                .map_err(|e| de::Error::custom(format!("Failed to parse '{normalized}' → {e}")))
        }

        fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_str(&v)
        }
    }

    deserializer.deserialize_any(NumberVisitor(std::marker::PhantomData))
}