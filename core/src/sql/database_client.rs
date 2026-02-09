use sqlx::Row;
use std::fmt::Debug;

#[derive(Debug)]
pub enum DbError {
    Sqlx(sqlx::Error),
    InvalidTable,
    NotFound,
    InvalidQuery(String),
}

#[async_trait::async_trait]
pub trait DatabaseClient: Send + Sync + Debug {
    async fn execute(&self, sql: &str) -> Result<(), DbError>;
    async fn execute_params(&self, sql: &str, params: &[&str]) -> Result<(), DbError>;

    async fn fetch_strings(&self, sql: &str) -> Result<Vec<String>, DbError>;
    async fn fetch_numbers(&self, sql: &str) -> Result<Vec<i64>, DbError>;
}

#[derive(Debug)]
pub struct MySqlClient {
    pub pool: sqlx::MySqlPool,
}

impl From<sqlx::Error> for DbError {
    fn from(err: sqlx::Error) -> Self {
        DbError::Sqlx(err)
    }
}

#[async_trait::async_trait]
impl DatabaseClient for MySqlClient {

    async fn execute_params(
        &self,
        sql: &str,
        params: &[&str],  // ← str reference
    ) -> Result<(), DbError> {
        let mut query = sqlx::query(sql);

        for param in params {
            query = query.bind(*param);
        }

        query.execute(&self.pool).await?;
        Ok(())
    }


    async fn execute(&self, sql: &str) -> Result<(), DbError> {
        sqlx::query(sql).execute(&self.pool).await?;
        Ok(())
    }

    async fn fetch_strings(&self, sql: &str) -> Result<Vec<String>, DbError> {
        let rows = sqlx::query(sql).fetch_all(&self.pool).await?;

        if rows.is_empty() {
            return Ok(vec![]);
        }
        Ok(rows
            .into_iter()
            .map(|row| {
                // dbg!(&row);
                row.get::<String, _>(0)
            })
            .collect())
    }

    async fn fetch_numbers(&self, sql: &str) -> Result<Vec<i64>, DbError> {
        let rows = sqlx::query(sql).fetch_all(&self.pool).await?;

        if !rows.is_empty() {
            return Ok(vec![]);
        }
        Ok(rows
            .into_iter()
            .map(|row| {
                // dbg!(&row);
                row.get::<i64, _>(0)
            })
            .collect())
    }
}

#[derive(Debug)]
pub struct SqliteClient {
    pub pool: sqlx::SqlitePool,
}

#[async_trait::async_trait]
impl DatabaseClient for SqliteClient {

    async fn execute_params(
        &self,
        sql: &str,
        params: &[&str],  // ← str reference
    ) -> Result<(), DbError> {
        let mut query = sqlx::query(sql);

        for param in params {
            query = query.bind(*param);
        }

        query.execute(&self.pool).await?;
        Ok(())
    }

    async fn execute(&self, sql: &str) -> Result<(), DbError> {
        sqlx::query(sql).execute(&self.pool).await?;
        Ok(())
    }

    async fn fetch_strings(&self, sql: &str) -> Result<Vec<String>, DbError> {
        let rows = sqlx::query(sql).fetch_all(&self.pool).await?;
        if rows.is_empty() {
            return Ok(vec![]);
        }
        Ok(rows
            .into_iter()
            .map(|row| row.get::<String, _>(0))
            .collect())
    }

    async fn fetch_numbers(&self, sql: &str) -> Result<Vec<i64>, DbError> {
        let rows = sqlx::query(sql).fetch_all(&self.pool).await?;
        if rows.is_empty() {
            return Ok(vec![]);
        }
        Ok(rows
            .into_iter()
            .map(|row| row.get::<i64, _>(0))
            .collect())
    }


}
