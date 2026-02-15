//! Small DX-friendly wrapper around `time 0.3`.
//!
//! Provides:
//! - Common `now()` helpers
//! - Precompiled format descriptions (no runtime parsing cost)
//! - Safe parsing helpers
//! - Compatible with PostgreSQL, MySQL and SQLite via `sqlx`
//!
//! All timestamps are UTC unless explicitly stated otherwise.

use time::{
    Date,
    OffsetDateTime,
    PrimitiveDateTime,
    error::Parse,
    format_description::FormatItem,
    format_description::well_known::Rfc2822,
    macros::format_description,
    format_description::parse
};

use serde::{Deserializer, Deserialize};
use serde::de::{Error};
/// =============================
/// Precompiled Format Definitions
/// =============================

/// Format: `YYYY-MM-DD`
/// Example: `2026-02-14`
const YMD: &[FormatItem<'static>] =
    format_description!("[year]-[month]-[day]");

/// Format: `YYYY-MM-DD HH:MM:SS`
/// Example: `2026-02-14 13:45:22`
const YMD_HMS: &[FormatItem<'static>] =
    format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");

/// Format: `YYYY_MM_DD_HHMM`
/// Example: `2026_02_14_1345`
const COMPACT: &[FormatItem<'static>] =
    format_description!("[year]_[month]_[day]_[hour][minute]");

/// =============================
/// NOW HELPERS (UTC)
/// =============================

/// Returns the current UTC time as `OffsetDateTime`.
///
/// Recommended for:
/// - Logging
/// - API responses
/// - `TIMESTAMP WITH TIME ZONE` columns
#[inline]
pub fn now() -> OffsetDateTime {
    OffsetDateTime::now_utc()
}

/// Returns current UTC date formatted as `YYYY-MM-DD`.
#[inline]
pub fn now_ymd() -> String {
    now().format(YMD).expect("valid YMD format")
}

/// Returns current UTC date-time formatted as `YYYY-MM-DD HH:MM:SS`.
#[inline]
pub fn now_ymd_hms() -> String {
    now().format(YMD_HMS).expect("valid YMD_HMS format")
}

/// Returns current UTC date-time formatted as `YYYY_MM_DD_HHMM`.
///
/// Useful for:
/// - File names
/// - Snapshot identifiers
/// - Backup naming
#[inline]
pub fn now_compact() -> String {
    now().format(COMPACT).expect("valid COMPACT format")
}

/// Returns current UTC time formatted as RFC2822.
///
/// Example:
/// `Sat, 14 Feb 2026 13:45:22 +0000`
#[inline]
pub fn now_rfc2822() -> String {
    now().format(&Rfc2822).expect("valid RFC2822 format")
}

/// =============================
/// PARSING HELPERS
/// =============================

/// Parses a string formatted as `YYYY-MM-DD` into `Date`.
///
/// # Example
/// ```
/// #  use rustavel_core::facades::datetime::parse_ymd;
/// let d = parse_ymd("2026-02-14").unwrap();
/// ```
#[inline]
pub fn parse_ymd(input: &str) -> Result<Date, Parse> {
    Date::parse(input, YMD)
}

/// Parses `YYYY-MM-DD HH:MM:SS` into `PrimitiveDateTime`.
///
/// This does NOT contain timezone information.
///
/// Recommended for:
/// - `TIMESTAMP WITHOUT TIME ZONE`
/// - MySQL `DATETIME`
#[inline]
pub fn parse_ymd_hms(input: &str) -> Result<PrimitiveDateTime, Parse> {
    PrimitiveDateTime::parse(input, YMD_HMS)
}

/// Parses `YYYY-MM-DD HH:MM:SS` and assumes UTC,
/// returning `OffsetDateTime`.
///
/// Use this if your database stores naive timestamps
/// but you treat them as UTC.
#[inline]
pub fn parse_ymd_hms_utc(
    input: &str,
) -> Result<OffsetDateTime, Parse> {
    let naive = PrimitiveDateTime::parse(input, YMD_HMS)?;
    Ok(naive.assume_utc())
}

/// Parses `YYYY_MM_DD_HHMM` into `PrimitiveDateTime`.
///
/// Useful for:
/// - Snapshot file names
/// - Custom compact identifiers
#[inline]
pub fn parse_compact(
    input: &str,
) -> Result<PrimitiveDateTime, Parse> {
    PrimitiveDateTime::parse(input, COMPACT)
}

/// Parses RFC2822 formatted string into `OffsetDateTime`.
///
/// Example:
/// `Sat, 14 Feb 2026 13:45:22 +0000`
#[inline]
pub fn parse_rfc2822(
    input: &str,
) -> Result<OffsetDateTime, Parse> {
    OffsetDateTime::parse(input, &Rfc2822)
}



pub fn deserialize_datetime<'de, D>(deserializer: D) -> Result<PrimitiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{

    let s: String = String::deserialize(deserializer)?;


    let s = s.trim();


    let format = format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");

    PrimitiveDateTime::parse(s, &format)
        .map_err(Error::custom)  // show 400 error
}

pub fn serialize_datetime<S>(date: &PrimitiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let format = format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
    let s = date
        .format(&format)
        .map_err(serde::ser::Error::custom)?;
    serializer.serialize_str(&s)
}