use std::{env, fmt::Display, str::FromStr};

/// Reads and parses an env var, falling back to `default` when unset.
/// Panics (fail-fast) on an invalid value so typos never go unnoticed.
pub fn env_parse<T>(key: &str, default: T) -> T
where
    T: FromStr,
    T::Err: Display,
{
    match env::var(key) {
        Ok(v) => v
            .trim()
            .parse()
            .unwrap_or_else(|e| panic!("{key} has an invalid value `{v}`: {e}")),
        Err(_) => default,
    }
}

pub fn env_bool(key: &str, default: bool) -> bool {
    match env::var(key) {
        Ok(v) => match v.trim().to_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => true,
            "0" | "false" | "no" | "off" => false,
            _ => panic!("{key} must be a boolean (true/false), got: {v}"),
        },
        Err(_) => default,
    }
}
