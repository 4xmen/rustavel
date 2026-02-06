use colored::*;
use std::time::Duration;
use terminal_size::{terminal_size, Width};

/// Available status results for an operation
pub enum Status {
    Done,
    Running,
    Failed,
}

/// Print a section title like:
/// INFO  Running migrations.
pub fn title(kind: TitleKind, message: &str) {
    let label = match kind {
        TitleKind::Info => " INFO ".on_blue().bold(),
        TitleKind::Warn => " WARN ".on_yellow().bold(),
        TitleKind::Error => " ERROR ".on_red().bold(),
        TitleKind::Success => " OK ".on_green().bold(),
    };

    println!("\n{}  {}", label, message.bold());
}

/// Print a timed operation line similar to Laravel output
pub fn operation(name: &str, duration: Duration, status: Status) {
    let mut term_width = terminal_width().unwrap_or(80) - 7;
    if term_width > 147 {
        term_width =  147;
    }

    let time_ms = duration.as_secs_f64() * 1000.0;
    let time_str = format!("{:.2}ms", time_ms);

    let status_str = match status {
        Status::Done => "DONE".green().bold(),
        Status::Failed => "FAILED".red().bold(),
        Status::Running => "Running".yellow().bold(),
    };

    // layout calculation
    let fixed_len =
        name.len() +
            1 +                 // space
            time_str.len() +
            1 +                 // space
            strip_ansi(&status_str.to_string()).len();

    let dots_len = term_width.saturating_sub(fixed_len).max(3);
    let dots = ".".repeat(dots_len);

    println!(
        " {} {} {} {}",
        name,
        dots.dimmed(),
        time_str.dimmed(),
        status_str
    );
}

/// Different title types
pub enum TitleKind {
    Info,
    Warn,
    Error,
    Success,
}

fn terminal_width() -> Option<usize> {
    terminal_size().map(|(Width(w), _)| w as usize)
}

/// Remove ANSI escape codes length effect
fn strip_ansi(input: &str) -> String {
    ansi_str::AnsiStr::ansi_strip(input).to_string()
}
