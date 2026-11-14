use crate::config::CONFIG;
use crate::config::database::DatabaseEngine;

/// is sqlite driver active
pub fn is_sqlite() -> bool {
    CONFIG.database.connection == DatabaseEngine::Sqlite
}

/// is postgres driver active
pub fn is_postgres() -> bool {
    CONFIG.database.connection == DatabaseEngine::PgSql
}
/// is postgres driver active
pub fn is_pgsql() -> bool {
    CONFIG.database.connection == DatabaseEngine::PgSql
}

/// is mysql driver active
pub fn is_mysql() -> bool {
    CONFIG.database.connection == DatabaseEngine::Mysql
}
