//! Small DX-friendly wrapper around `time 0.3`.
//!
//! Provides:
//! - Common `now()` helpers
//! - Precompiled format descriptions (no runtime parsing cost)
//! - Safe parsing helpers
//! - Compatible with PostgreSQL, MySQL and SQLite via `sqlx`
//!
//! All timestamps are UTC unless explicitly stated otherwise.

use std::fmt::Write;
use time::{
    Date,
    OffsetDateTime,
    PrimitiveDateTime,
    Time,
    error::Parse,
    format_description::FormatItem,
    format_description::well_known::Rfc2822,
    macros::format_description,
    Weekday,
    Month,
    // format_description::parse
};

use serde::{Deserializer, Deserialize};
use serde::de::{Error};
use crate::config::CONFIG;
use crate::localization::digits::{apply_normalized_string, normalize_digits};
use crate::localization::numbers::apply_normalize_number;
// use jalali_rs::gregorian_to_jalali;


#[derive(Clone, Copy)]
struct Jalali {
    year: i32,
    month: u8,
    day: u8,
}
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


/// Format: `HH:MM:SS`
/// Example: `13:45:22`
const HMS: &[FormatItem<'static>] =
    format_description!("[hour]:[minute]:[second]");


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


/// Returns the current date and time as a PrimitiveDateTime.
pub fn now_primitive() -> PrimitiveDateTime {
    // Get the current time as an OffsetDateTime (which includes timezone info).
    let current_time = OffsetDateTime::now_utc();
    // Convert to PrimitiveDateTime by using date() and time().
    PrimitiveDateTime::new(current_time.date(), current_time.time())
}


/// convert string to PrimitiveDateTime
pub fn parse_to_primitive_datetime(datetime_str: &str) -> Result<PrimitiveDateTime, time::Error> {
    // Parse the string into a PrimitiveDateTime.
    Ok(PrimitiveDateTime::parse(datetime_str, &YMD_HMS)?)
}

pub fn deserialize_datetime<'de, D>(deserializer: D) -> Result<PrimitiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{

    let s: String = String::deserialize(deserializer)?;
    let s = normalize_digits(s.trim());

    PrimitiveDateTime::parse(&s, &YMD_HMS)
        .map_err(Error::custom)  // show 400 error
}

pub fn serialize_datetime<S>(date: &PrimitiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let s = date
        .format(&YMD_HMS)
        .map_err(serde::ser::Error::custom)?;
    serializer.serialize_str(&s)
}

pub fn deserialize_date<'de, D>(deserializer: D) -> Result<Date, D::Error>
where
    D: Deserializer<'de>,
{

    let s: String = String::deserialize(deserializer)?;
    let s = normalize_digits(s.trim());

    Date::parse(&s, &YMD)
        .map_err(Error::custom)  // show 400 error
}

pub fn serialize_date<S>(date: &Date, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let s = date
        .format(&YMD)
        .map_err(serde::ser::Error::custom)?;
    serializer.serialize_str(&s)
}


pub fn deserialize_time<'de, D>(deserializer: D) -> Result<Time, D::Error>
where
    D: Deserializer<'de>,
{

    let s: String = String::deserialize(deserializer)?;

    let s = s.trim();

    Time::parse(s, &HMS)
        .map_err(Error::custom)  // show 400 error
}

pub fn serialize_time<S>(time: &Time, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let s = time
        .format(&HMS)
        .map_err(serde::ser::Error::custom)?;
    serializer.serialize_str(&s)
}




/// Returns true if year is leap year (Gregorian)
#[inline]
fn is_leap(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}


#[inline]
fn jdate_imp(dt: OffsetDateTime, format: &str) -> String {

    let gy = dt.year();
    let gm = dt.month() as u8;
    let gd = dt.day();

    let j = gregorian_to_jalali(gy, gm, gd);

    let hour = dt.hour();
    let minute = dt.minute();
    let second = dt.second();

    let is_pm = hour >= 12;

    let mut out = String::with_capacity(format.len() + 16);
    let mut chars = format.chars();

    while let Some(ch) = chars.next() {
        match ch {
            // Jalali date
            'Y' => out.push_str(&j.year.to_string()),
            'y' => out.push_str(&(j.year % 100).to_string()),

            'm' => push_2(&mut out, j.month),
            'n' => out.push_str(&j.month.to_string()),

            'd' => push_2(&mut out, j.day),
            'j' => out.push_str(&j.day.to_string()),

            // month names (basic Persian/English placeholder)
            'F' => out.push_str(jalali_month_name(j.month)),
            'M' => out.push_str(&jalali_month_name(j.month)[..3]),

            // weekday still Gregorian (if needed)
            'H' => push_2(&mut out, hour),
            'i' => push_2(&mut out, minute),
            's' => push_2(&mut out, second),

            'a' => out.push_str(if is_pm { "pm" } else { "am" }),
            'A' => out.push_str(if is_pm { "PM" } else { "AM" }),

            // fallback
            other => out.push(other),
        }
    }

    out
}

#[inline]
fn gregorian_to_jalali(mut gy: i32,  gm: u8,  gd: u8) -> Jalali {
    let g_d_m = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];

    let mut jy = if gy > 1600 {
        979
    } else {
        0
    };

    gy -= 1600;

    let gy2 = if gm > 2 { gy + 1 } else { gy };

    let days =
        365 * gy
            + (gy2 + 3) / 4
            - (gy2 + 99) / 100
            + (gy2 + 399) / 400
            - 80
            + gd as i32
            + g_d_m[(gm - 1) as usize];

    jy += 33 * (days / 12053);
    let mut days = days % 12053;

    jy += 4 * (days / 1461);
    days %= 1461;

    if days > 365 {
        jy += (days - 1) / 365;
        days = (days - 1) % 365;
    }

    let jm = if days < 186 {
        1 + (days / 31)
    } else {
        7 + ((days - 186) / 30)
    };

    let jd = if days < 186 {
        1 + (days % 31)
    } else {
        1 + ((days - 186) % 30)
    };

    Jalali {
        year: jy,
        month: jm as u8,
        day: jd as u8,
    }
}

#[inline(always)]
fn push_2(out: &mut String, n: u8) {
    out.push((b'0' + (n / 10)) as char);
    out.push((b'0' + (n % 10)) as char);
}

#[inline]
fn format_php_imp(dt: OffsetDateTime, format: &str) -> String {

    let year = dt.year();
    let month = dt.month();
    let day = dt.day();
    let weekday = dt.weekday();
    let ordinal = dt.ordinal(); // z
    let hour = dt.hour();
    let minute = dt.minute();
    let second = dt.second();
    let nanos = dt.nanosecond();

    let is_pm = hour >= 12;

    let mut out = String::with_capacity(format.len() + 16);

    let mut chars = format.chars();

    while let Some(ch) = chars.next() {
        match ch {
            // day
            'd' => out.push_str(&format!("{:02}", day)),
            'j' => out.push_str(&day.to_string()),
            'D' => out.push_str(match weekday {
                Weekday::Monday => "Mon",
                Weekday::Tuesday => "Tue",
                Weekday::Wednesday => "Wed",
                Weekday::Thursday => "Thu",
                Weekday::Friday => "Fri",
                Weekday::Saturday => "Sat",
                Weekday::Sunday => "Sun",
            }),
            'l' => out.push_str(match weekday {
                Weekday::Monday => "Monday",
                Weekday::Tuesday => "Tuesday",
                Weekday::Wednesday => "Wednesday",
                Weekday::Thursday => "Thursday",
                Weekday::Friday => "Friday",
                Weekday::Saturday => "Saturday",
                Weekday::Sunday => "Sunday",
            }),
            'N' => out.push_str(&((weekday.number_from_monday()).to_string())),
            'w' => out.push_str(&(weekday.number_from_sunday().to_string())),
            'z' => out.push_str(&ordinal.to_string()),

            // month
            'F' => out.push_str(&month.to_string()),
            'm' => out.push_str(&format!("{:02}", u8::from(month))),
            'M' => out.push_str(month.to_string().get(0..3).unwrap_or("")),
            'n' => out.push_str(&u8::from(month).to_string()),
            't' => {
                let days = match month {
                    Month::January => 31,
                    Month::February => {
                        if is_leap(year) { 29 } else { 28 }
                    }
                    Month::March => 31,
                    Month::April => 30,
                    Month::May => 31,
                    Month::June => 30,
                    Month::July => 31,
                    Month::August => 31,
                    Month::September => 30,
                    Month::October => 31,
                    Month::November => 30,
                    Month::December => 31,
                };
                out.push_str(&days.to_string());
            }
            'L' => out.push_str(if is_leap(year) { "1" } else { "0" }),

            // year
            'Y' => out.push_str(&year.to_string()),
            'y' => out.push_str(&(year % 100).to_string()),

            // time
            'a' => out.push_str(if is_pm { "pm" } else { "am" }),
            'A' => out.push_str(if is_pm { "PM" } else { "AM" }),
            'g' => {
                let h = hour % 12;
                let h = if h == 0 { 12 } else { h };
                out.push_str(&h.to_string());
            }
            'G' => out.push_str(&hour.to_string()),
            'h' => {
                let h = hour % 12;
                let h = if h == 0 { 12 } else { h };
                out.push_str(&format!("{:02}", h));
            }
            'H' => out.push_str(&format!("{:02}", hour)),
            'i' => out.push_str(&format!("{:02}", minute)),
            's' => out.push_str(&format!("{:02}", second)),

            // fractions
            'u' => out.push_str(&format!("{:06}", nanos / 1000)), // microseconds
            'v' => out.push_str(&format!("{:03}", nanos / 1_000_000)), // milliseconds

            // ISO-like
            'c' => {
                out.push_str(&dt.date().to_string());
                out.push('T');
                out.push_str(&format!("{:02}:{:02}:{:02}", hour, minute, second));
            }

            'r' => {
                out.push_str(&format!(
                    "{}, {:02} {} {} {:02}:{:02}:{:02}",
                    weekday,
                    day,
                    month,
                    year,
                    hour,
                    minute,
                    second
                ));
            }

            // fallback
            other => out.push(other),
        }
    }

    out
}

fn jalali_month_name(m: u8) -> &'static str {
    match m {
        1 => "فروردین",
        2 => "اردیبهشت",
        3 => "خرداد",
        4 => "تیر",
        5 => "مرداد",
        6 => "شهریور",
        7 => "مهر",
        8 => "آبان",
        9 => "آذر",
        10 => "دی",
        11 => "بهمن",
        12 => "اسفند",
        _ => "",
    }
}

#[inline]
fn diff_for_humans_impl(datetime: OffsetDateTime) -> String {
    let now = OffsetDateTime::now_utc();

    let diff = now - datetime;

    let future = diff.is_negative();

    // unsigned_abs avoids branching and negative handling cost
    let secs = diff.whole_seconds().unsigned_abs();

    // Carbon-style thresholds
    const MINUTE: u64 = 60;
    const HOUR: u64 = 60 * MINUTE;
    const DAY: u64 = 24 * HOUR;
    const WEEK: u64 = 7 * DAY;
    const MONTH: u64 = 30 * DAY;
    const YEAR: u64 = 365 * DAY;

    // Small fast-paths first
    if secs < 5 {
        return if future {
            "in a few seconds".into()
        } else {
            "just now".into()
        };
    }

    let (value, unit): (u64, &'static str) = match secs {
        5..=44 => (secs, "second"),

        45..=89 => (1, "minute"),

        90..=2_699 => (secs / MINUTE, "minute"),

        2_700..=5_399 => (1, "hour"),

        5_400..=86_399 => (secs / HOUR, "hour"),

        86_400..=172_799 => (1, "day"),

        172_800..=604_799 => (secs / DAY, "day"),

        604_800..=1_209_599 => (1, "week"),

        1_209_600..=2_591_999 => (secs / WEEK, "week"),

        2_592_000..=5_183_999 => (1, "month"),

        5_184_000..=31_535_999 => (secs / MONTH, "month"),

        31_536_000..=63_071_999 => (1, "year"),

        _ => (secs / YEAR, "year"),
    };

    // Pre-allocate enough capacity for almost all outputs
    let mut out = String::with_capacity(32);

    if future {
        out.push_str("in ");
    }

    // Faster than format!
    let _ = write!(&mut out, "{} {}", value, unit);

    if value != 1 {
        out.push('s');
    }

    if !future {
        out.push_str(" ago");
    }

    out
}


/// Extension methods for formatting and displaying dates.
///
/// This trait provides localized date formatting utilities,
/// including Jalali (Shamsi) calendar support and human-readable
/// relative time formatting similar to Laravel Carbon.
pub trait TimeExt {



    /// Formats the date as a Jalali (Shamsi) date string.
    ///
    /// The `format` argument follows the crate's supported
    /// formatting syntax.
    ///
    /// # Example
    ///
    /// ```rust
    /// let formatted = datetime.jdate("Y/m/d");
    /// ```
    fn jdate(&self, format: &str) -> String;

    /// Formats the date based on the current locale.
    ///
    /// For Persian locales, the date is formatted using
    /// the Jalali (Shamsi) calendar. For other locales,
    /// the Gregorian calendar is used.
    ///
    /// The `format` argument follows the crate's supported
    /// formatting syntax.
    ///
    /// # Example
    ///
    /// ```rust
    /// let formatted = datetime.ldate("Y-m-d");
    /// ```
    fn ldate(&self, format: &str) -> String;

    /// Returns a human-readable relative time string.
    ///
    /// Similar to Laravel Carbon's `diffForHumans`,
    /// this method generates expressions such as:
    ///
    /// - `"2 minutes ago"`
    /// - `"3 days ago"`
    /// - `"in 1 hour"`
    /// - `"just now"`
    ///
    /// The comparison is typically made against the current time.
    fn diff_for_humans(&self) -> String;

    /// Formats the datetime using PHP-style format tokens.
    ///
    /// This function mimics PHP's `date()` behavior with a subset of
    /// supported tokens such as:
    ///
    /// - `Y` → 4-digit year
    /// - `m` → zero-padded month
    /// - `d` → zero-padded day
    /// - `H` → 24-hour format
    /// - `i` → minutes
    /// - `s` → seconds
    ///
    /// # Performance
    ///
    /// This method is designed for high-performance scenarios:
    /// - Avoids regex parsing
    /// - Uses single-pass iteration
    /// - Suitable for frequent calls (e.g. logging, serialization)
    ///
    /// # Notes
    ///
    /// - Unsupported tokens may be ignored or passed through literally
    /// - Assumes Gregorian calendar unless otherwise specified
    fn format_php(&self, format: &str) -> String ;
}


impl TimeExt for PrimitiveDateTime {

    fn jdate(&self, format: &str) -> String {
        let datetime = self.assume_utc();

        jdate_imp(datetime,format)
    }
    fn ldate(&self, format: &str) -> String {
        let datetime = self.assume_utc();
        println!("what: {}",CONFIG.app.locale);
        if CONFIG.app.locale == "fa" {
            jdate_imp(datetime,format)
        }else{
            format_php_imp(datetime,format)
        }
    }
    fn diff_for_humans(&self) -> String {
        // PrimitiveDateTime has no timezone information.
        // We assume UTC to make comparison possible.
        let datetime = self.assume_utc();

        diff_for_humans_impl(datetime)
    }

    fn format_php(&self, format: &str) -> String {
        let datetime = self.assume_utc();

        format_php_imp(datetime,format)
    }
}

impl TimeExt for OffsetDateTime {


    fn jdate(&self, format: &str) -> String {
       jdate_imp(*self,format)
    }
    fn ldate(&self, format: &str) -> String {
        if CONFIG.app.locale == "fa" {
            jdate_imp(*self,format)
        }else{
            format_php_imp(*self,format)
        }
    }
    fn diff_for_humans(&self) -> String {
        diff_for_humans_impl(*self)
    }
    fn format_php(&self, format: &str) -> String {

        format_php_imp(*self,format)
    }
}
impl TimeExt for Date {
    fn jdate(&self, format: &str) -> String {
        let datetime = PrimitiveDateTime::new(
            *self,
            Time::MIDNIGHT,
        );

        datetime.jdate(format)
    }
    fn ldate(&self, format: &str) -> String {

        let datetime = PrimitiveDateTime::new(
            *self,
            Time::MIDNIGHT,
        );
        if CONFIG.app.locale == "fa" {
            datetime.jdate(format)
        }else{
            datetime.format_php(format)
        }

    }
    fn diff_for_humans(&self) -> String {
        let datetime = PrimitiveDateTime::new(
            *self,
            Time::MIDNIGHT,
        );

        datetime.diff_for_humans()
    }

    fn format_php(&self, format: &str) -> String {

        let datetime = PrimitiveDateTime::new(
            *self,
            Time::MIDNIGHT,
        );

        datetime.format_php(format)
    }
}


