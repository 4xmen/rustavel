use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub name: String,
    pub env: String,
    pub debug: bool,
    pub host: String,
    pub port: u16,
    pub timezone: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            name: "Rustavel".into(),
            env: "production".into(),
            debug: false,
            host: "127.0.0.1".into(),
            port: 3000,
            timezone: "UTC".into(),
        }
    }
}

impl AppConfig {
    pub fn from_env() -> Self {
        let mut cfg = Self::default();

        if let Ok(v) = env::var("APP_NAME") {
            cfg.name = v;
        }

        if let Ok(v) = env::var("APP_ENV") {
            cfg.env = v;
        }

        if let Ok(v) = env::var("APP_DEBUG") {
            cfg.debug = v == "true" || v == "1";
        }

        if let Ok(v) = env::var("APP_IP") {
            cfg.host = v;
        }

        if let Ok(v) = env::var("APP_PORT") {
            cfg.port = v.parse().expect("APP_PORT must be a number");
        }

        if let Ok(v) = env::var("APP_TIMEZONE") {
            cfg.timezone = v;
        }

        cfg
    }
}
