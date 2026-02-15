use std::collections::HashMap;
use serde::Serialize;
use regex::Regex;
use once_cell::sync::Lazy;
use time::macros::datetime;

use time::{
    PrimitiveDateTime,
    format_description::{self, FormatItem},
};

use serde_json::de::IoRead;
use serde_json::Deserializer;
use std::net::IpAddr;

pub static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap()
});

pub static URL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(https?|ftps?)://[^\s/$.?#].[^\s]*$").unwrap()
});

pub static HEX_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^#?([a-fA-F0-9]{3}|[a-fA-F0-9]{6})$").unwrap()
});


static DATE_FORMAT: Lazy<Vec<FormatItem>> = Lazy::new(|| {
    format_description::parse("[year]-[month]-[day]").unwrap()
});
#[derive(Debug,Serialize)]
pub struct ValidationErrors {
    pub errors: HashMap<String, Vec<String>>,
}

impl ValidationErrors {
    pub fn new() -> Self {
        Self {
            errors: HashMap::new(),
        }
    }

    pub fn add(&mut self, field: impl Into<String>, message: impl Into<String>) {
        self.errors
            .entry(field.into())
            .or_default()
            .push(message.into());
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }
}

pub trait CheckMateValidator : Sync + Send {
    fn validate(&self) -> Result<(), ValidationErrors>;
}

/// Validate if the given string is a properly formatted email address.
///
/// This function uses a regular expression to check that the input string matches
/// the general format of an email address. The format requires that the email
/// consists of alphanumeric characters, followed by an '@' symbol, then a domain
/// name with at least one dot separating the domain and the top-level domain.
///
/// # Example
///
/// ```
/// # use macros_core::is_valid_email;
/// assert_eq!(is_valid_email("example@example.com"), true);
/// assert_eq!(is_valid_email("invalid-email"), false);
/// ```
pub fn is_valid_email(value: &str) -> bool {

    EMAIL_REGEX.is_match(value)
}

/// Validate if the given string is a properly formatted URL.
///
/// This function checks if the input string matches the general format for URLs.
/// It supports schemes such as `http`, `https`, `ftp`, and `ftps`. The URL
/// must start with one of these schemes followed by `://`, and include a valid
/// domain as well as optional paths.
///
/// # Example
///
/// ```
/// # use macros_core::is_valid_url;
/// assert_eq!(is_valid_url("http://example.com"), true);
/// assert_eq!(is_valid_url("ftp://files.example.com/resource"), true);
/// assert_eq!(is_valid_url("invalid-url"), false);
/// ```
pub fn is_valid_url(value: &str) -> bool {
    URL_REGEX.is_match(value)
}




/// Validate if the given string is a valid hexadecimal color.
///
/// Supports formats like:
/// - #FFF
/// - #FFFFFF
/// - FFF
/// - FFFFFF
pub fn is_valid_hex_color(value: &str) -> bool {

    HEX_REGEX.is_match(value)
}

/// Returns `true` if all characters in the string are ASCII.
///
/// This function checks whether every byte in the string is within the
/// ASCII range (0x00..=0x7F).
///
/// # Examples
///
/// ```
/// # use macros_core::is_valid_ascii;
/// assert!(is_valid_ascii("Hello"));
/// assert!(!is_valid_ascii("سلام"));
/// ```
pub fn is_valid_ascii(s: &str) -> bool {
    s.is_ascii()
}


/// Returns `true` if all characters in the string are ASCII alphanumeric.
///
/// This function checks whether every character in the string is within the
/// ASCII alphanumeric range: `A-Z`, `a-z`, or `0-9`.
///
/// # Examples
///
/// ```
/// # use macros_core::is_valid_ascii_alphanumeric;
/// assert!(is_valid_ascii_alphanumeric("Hello123"));
/// assert!(!is_valid_ascii_alphanumeric("Hello_123"));
/// assert!(!is_valid_ascii_alphanumeric("سلام"));
/// ```
pub fn is_valid_ascii_alphanumeric(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_alphanumeric())
}

/// Returns `true` if the string is a valid IPv4 or IPv6 address.
///
/// This function tries to parse the string into an `IpAddr`.
/// If parsing succeeds, the string is a valid IP address.
///
/// # Examples
///
/// ```
/// # use macros_core::is_valid_ip;
/// assert!(is_valid_ip("192.168.0.1"));
/// assert!(is_valid_ip("::1"));
/// assert!(!is_valid_ip("999.999.999.999"));
/// assert!(!is_valid_ip("not_an_ip"));
/// ```
pub fn is_valid_ip(s: &str) -> bool {
    s.parse::<IpAddr>().is_ok()
}

/// Checks whether the input is valid JSON.
/// Input: value `&str`
/// Output: `true` if the string is a complete, valid JSON value; otherwise `false`.
/// Behavior: no panics, high-performance, idiomatic.
pub fn is_valid_json(value: &str) -> bool {
    // Use Deserializer to parse a single JSON value and ensure no trailing non-whitespace data.
    // This approach gives more control and can be faster than `serde_json::from_str` for this check.
    if serde_json::from_str::<serde_json::Value>(value).is_err() {
       return  false;
    }

    true
}
/// Validate if the given string is a valid date (YYYY-MM-DD).
pub fn is_valid_date(value: &str) -> bool {
    is_valid_php_datetime("Y-m-d",value)
}

/// Validate if the given string is a valid datetime (YYYY-MM-DD HH:MM:SS).
pub fn is_valid_datetime(value: &str) -> bool {
    is_valid_php_datetime("Y-m-d H:i:s",value)
}

/// Validate if the given string is a valid time (HH:MM:SS).
pub fn is_valid_time(value: &str) -> bool {
    is_valid_php_datetime("H:i:s",value)
}

/// Validate if `value` is after the given date (YYYY-MM-DD).
pub fn is_after(value: &str, target: &str) -> bool {
    let value_date = PrimitiveDateTime::parse(value, &*DATE_FORMAT).unwrap();
    let target_date = PrimitiveDateTime::parse(target, &*DATE_FORMAT).unwrap();
    value_date > target_date
}

/// Validate if `value` is before the given date (YYYY-MM-DD).
pub fn is_before(value: &str, target: &str) -> bool {
    let value_date = PrimitiveDateTime::parse(value, &*DATE_FORMAT).unwrap();
    let target_date = PrimitiveDateTime::parse(target, &*DATE_FORMAT).unwrap();
    value_date < target_date
}


pub fn is_after_option(value: &str, target: &str) -> Option<bool> {
    let value_date = PrimitiveDateTime::parse(value, &*DATE_FORMAT).ok()?;
    let target_date = PrimitiveDateTime::parse(target, &*DATE_FORMAT).ok()?;
    Some(value_date > target_date)
}


pub fn is_before_option(value: &str, target: &str) -> Option<bool> {
    let value_date = PrimitiveDateTime::parse(value, &*DATE_FORMAT).ok()?;
    let target_date = PrimitiveDateTime::parse(target, &*DATE_FORMAT).ok()?;
    Some(value_date < target_date)
}


/// Validates a datetime string using a PHP-style format.
///
/// # Arguments
/// - `php_format`: PHP-style format string (e.g. "Y/m/d H:i:s")
/// - `input`: datetime string to validate
///
/// # Returns
/// - `true` if parsing succeeds
/// - `false` if parsing fails
///
/// # Example
/// ```
/// # use macros_core::is_valid_php_datetime;
/// assert!(is_valid_php_datetime("Y/m/d H:i:s", "2026/02/14 13:45:22"));
/// assert!(!is_valid_php_datetime("Y/m/d H:i:s", "invalid"));
/// ```
pub fn is_valid_php_datetime(
    php_format: &str,
    input: &str,
) -> bool {
    // Convert PHP format to time format string
    let converted = match convert_php_to_time_format(php_format) {
        Some(f) => f,
        None => return false,
    };

    // Parse format description inside same scope
    let format = match format_description::parse(&converted) {
        Ok(f) => f,
        Err(_) => return false,
    };

    PrimitiveDateTime::parse(input, &format).is_ok()
}


/// Converts a limited subset of PHP date format tokens
/// into `time` crate format description.
///
/// This does NOT support all PHP tokens.
/// Only common numeric tokens are handled.
fn convert_php_to_time_format(php_format: &str) -> Option<String> {
    let mut converted = String::new();

    for ch in php_format.chars() {
        match ch {
            // Year
            'Y' => converted.push_str("[year]"),
            'y' => converted.push_str("[year repr:last_two]"),

            // Month
            'm' => converted.push_str("[month]"),
            'n' => converted.push_str("[month padding:none]"),

            // Day
            'd' => converted.push_str("[day]"),
            'j' => converted.push_str("[day padding:none]"),

            // Hour (24h)
            'H' => converted.push_str("[hour]"),
            'G' => converted.push_str("[hour padding:none]"),

            // Minute
            'i' => converted.push_str("[minute]"),

            // Second
            's' => converted.push_str("[second]"),

            // Allowed literal characters
            '/' | '-' | ':' | ' ' => converted.push(ch),

            // Anything else → unsupported
            _ => return None,
        }
    }

    Some(converted)
}