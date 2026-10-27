// Bugs 31, 32, 33, 34 and 35
use rustavel_core::config::CONFIG;
use rustavel_core::facades::datetime::TimeExt;
use rustavel_core::localization::digits::normalize_digits;
use time::macros::datetime;

#[test]
fn bug31_jdate_month_short_name() {
    let r = std::panic::catch_unwind(|| datetime!(2026-10-22 15:00:00 UTC).jdate("M"));
    assert!(r.is_ok(), "jdate(\"M\") panicked");
}

#[test]
fn bug32_persian_am_pm() {
    if CONFIG.app.timezone != "UTC" { return; }
    assert_eq!(datetime!(2026-10-22 15:00:00 UTC).jdate("a"), "ب‌ظ");
    assert_eq!(datetime!(2026-10-22 09:00:00 UTC).jdate("a"), "ق‌ظ");
}

#[test]
fn bug33_normalize_digits() {
    assert_eq!(normalize_digits("https://x.com/a"), "https://x.com/a");
    assert_eq!(normalize_digits("۱۴۰۴/۰۱/۰۱"), "1404/01/01");
    assert_eq!(normalize_digits("۳٫۵"), "3.5");
}

#[test]
fn bug34_format_php_like_php() {
    if CONFIG.app.timezone != "UTC" { return; }
    assert_eq!(datetime!(2026-10-25 10:00:00 UTC).format_php("w"), "0", "Sunday");
    assert_eq!(datetime!(2026-01-01 10:00:00 UTC).format_php("z"), "0", "1 January");
    assert_eq!(datetime!(2005-01-01 10:00:00 UTC).format_php("y"), "05");
    assert_eq!(datetime!(2026-10-22 15:04:05 UTC).format_php("r"), "Thu, 22 Oct 2026 15:04:05 +0000");
}

#[test]
fn bug35_app_timezone_is_used() {
    // run with APP_TIMEZONE=+03:30 (see the guide)
    if CONFIG.app.timezone != "+03:30" { eprintln!("skipped: run with APP_TIMEZONE=+03:30"); return; }
    // 22:00 UTC is 01:30 the next day in Tehran
    assert_eq!(datetime!(2026-10-22 22:00:00 UTC).jdate("Y-m-d H:i"), "1405-08-01 01:30");
}