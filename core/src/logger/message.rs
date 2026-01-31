
#[allow(dead_code)]
#[derive(Debug)]
pub enum LogLevel {
    Log,
    Debug,
    Info,
    Warn,
    Error,
    Success,
}

#[derive(Debug)]
pub struct LogMessage {
    pub level: LogLevel,
    pub message: String,
    pub timestamp: u128,
}
