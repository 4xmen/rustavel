use std::env;

#[derive(Debug, Clone)]
pub enum DatabaseEngine {
    Mysql,
    Sqlite,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub connection: DatabaseEngine,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
    pub prefix: String,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            connection: DatabaseEngine::Mysql,
            host: "127.0.0.1".into(),
            port: 3306,
            username: "root".into(),
            password: "".into(),
            database: "test".into(),
            prefix: "".into(),
        }
    }
}

impl DatabaseConfig {
    pub fn from_env() -> Self {
        let mut cfg = Self::default();

        if let Ok(v) = env::var("DB_CONNECTION") {
            if let Some(connection) = DatabaseEngine::from_str(&v) {
                cfg.connection = connection;
            } else {
                eprintln!("Invalid DB_CONNECTION value: {}", v);
            }
        }

        if let Ok(v) = env::var("DB_HOST") {
            cfg.host = v;
        }

        if let Ok(v) = env::var("DB_PORT") {
            cfg.port = v.parse().expect("DB_PORT must be a number");
        }

        if let Ok(v) = env::var("DB_USERNAME") {
            cfg.username = v;
        }

        if let Ok(v) = env::var("DB_PASSWORD") {
            cfg.password = v;
        }

        if let Ok(v) = env::var("DB_DATABASE") {
            cfg.database = v;
        }

        if let Ok(v) = env::var("DB_PREFIX") {
            cfg.prefix = v;
        }

        cfg
    }
}

impl DatabaseEngine {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "mysql" => Some(DatabaseEngine::Mysql),
            "sqlite" => Some(DatabaseEngine::Sqlite),
            _ => None,
        }
    }
}
