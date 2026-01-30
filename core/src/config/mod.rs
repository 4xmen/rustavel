use once_cell::sync::Lazy;

pub mod app;
pub mod database;

#[derive(Debug)]
pub struct Config {
    pub app: app::AppConfig,
    pub database: database::DatabaseConfig,
    // pub cache: CacheConfig, [sample how to add another one next time]
}

impl Config {
    fn load() -> Self {
        dotenv::dotenv().ok();

        Self {
            app: app::AppConfig::from_env(),
            database: database::DatabaseConfig::from_env(),
            // cache: CacheConfig::from_env(), [sample how to add another one next time]
        }
    }
}

pub static CONFIG: Lazy<Config> = Lazy::new(|| Config::load());
