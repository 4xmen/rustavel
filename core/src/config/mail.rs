use std::env;

#[derive(Debug, Clone, PartialEq)]
pub enum MailDriver {
    Stmp,
    SendMail,
    Log,
    Array,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MailEncryptType {
    Tls,
    Ssl,
    None,
}
#[derive(Debug, Clone)]
pub struct MailConfig {
    pub mailer: MailDriver,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub encryption: MailEncryptType,
    pub from_address: String,
    pub from_name: String,
}

impl Default for MailConfig {
    fn default() -> Self {
        Self {
            mailer: MailDriver::SendMail,
            host: "smtp.example.com".into(),
            port: 587,
            username: "smtp.example.com".into(),
            password: "secret".into(),
            encryption: MailEncryptType::Tls,
            from_address: "smtp.example.com".into(),
            from_name: "rustavel".into(),
        }
    }
}

impl MailConfig {
    pub fn from_env() -> Self {
        let mut cfg = Self::default();

        if let Ok(v) = env::var("MAIL_MAILER") {
            if let Some(mailer) = MailDriver::from_str(&v) {
                cfg.mailer = mailer;
            } else {
                eprintln!("Invalid MAIL_MAILER value: {}", v);
            }
        }

        if let Ok(v) = env::var("MAIL_HOST") {
            cfg.host = v;
        }

        if let Ok(v) = env::var("MAIL_PORT") {
            cfg.port = v.parse().expect("APP_PORT must be a number");
        }

        if let Ok(v) = env::var("MAIL_USERNAME") {
            cfg.username = v;
        }

        if let Ok(v) = env::var("MAIL_PASSWORD") {
            cfg.password = v;
        }

        if let Ok(v) = env::var("MAIL_ENCRYPTION") {
            if let Some(encryption) = MailEncryptType::from_str(&v) {
                cfg.encryption = encryption;
            } else {
                eprintln!("Invalid MAIL_ENCRYPTION value: {}", v);
            }
        }
        if let Ok(v) = env::var("MAIL_FROM_ADDRESS") {
            cfg.from_address = v;
        }

        if let Ok(v) = env::var("MAIL_FROM_NAME") {
            cfg.from_name = v;
        }
        cfg
    }
}

impl MailDriver {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "smpt" => Some(MailDriver::Stmp),
            "sendmail" => Some(MailDriver::SendMail),
            "log" => Some(MailDriver::Log),
            "array" => Some(MailDriver::Array),
            _ => None,
        }
    }
}
impl MailEncryptType {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "ssl" => Some(MailEncryptType::Ssl),
            "tls" => Some(MailEncryptType::Tls),
            _ => Some(MailEncryptType::None)
        }
    }
}
