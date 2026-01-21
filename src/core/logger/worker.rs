use std::fs::{File, OpenOptions};
use std::io::Write;

use tokio::sync::mpsc::UnboundedReceiver;

use super::message::{LogLevel, LogMessage};

pub struct LoggerWorker {
    file: Option<File>,
    rx: UnboundedReceiver<LogMessage>,
}

impl LoggerWorker {
    pub fn new(rx: UnboundedReceiver<LogMessage>) -> Self {
        let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open("storage/logs/rustavel.log")
            .ok();

        Self { file, rx }
    }

    pub async fn run(mut self) {
        while let Some(msg) = self.rx.recv().await {
            let line = format!(
                "{} [{}] {}",
                msg.timestamp,
                Self::level_str(&msg.level),
                msg.message
            );

            match msg.level {
                LogLevel::Error =>  println!("\x1b[31m{line}\x1b[0m"),
                LogLevel::Warn =>  println!("\x1b[33m{line}\x1b[0m"),
                LogLevel::Success =>  println!("\x1b[32m{line}\x1b[0m"),
                LogLevel::Info =>  println!("\x1b[36m{line}\x1b[0m"),
                _ =>  println!("{line}")
            };

            if let Some(file) = &mut self.file {
                let _ = writeln!(file, "{line}");
            }
        }
    }

    fn level_str(level: &LogLevel) -> &'static str {
        match level {
            LogLevel::Log => "LOG",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Success => "SUCCESS",
        }
    }
}
