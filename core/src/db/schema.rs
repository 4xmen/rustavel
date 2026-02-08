use std::collections::HashMap;
use crate::config::CONFIG;
use crate::config::database::DatabaseEngine;
use illuminate_string::Str;
use crate::logger;
use crate::sql::database_client::{DatabaseClient, DbError, MySqlClient, SqliteClient};
use crate::sql::generator::SqlGenerator;
use crate::sql::mysql::MySqlGenerator;
use crate::sql::sqlite::SqliteGenerator;
use crate::db::table::{Table, TableAction};
use sqlx::{MySqlPool, SqlitePool};
use tokio::time::Instant;
use crate::facades::terminal_ui::{Status,operation};

#[derive(Debug)]
pub struct Schema {
    prefix: String,
    generator: Box<dyn SqlGenerator + Send + Sync>,
    client: Box<dyn DatabaseClient + Send + Sync>,
    debug: bool,
    tables: HashMap<String, Table>,
    current: Option<Table>,
}

impl Schema {
    pub async fn new() -> Result<Self, sqlx::Error> {
        let debug = CONFIG.app.debug;

        let (generator, client): (Box<dyn SqlGenerator>, Box<dyn DatabaseClient>) =
            match CONFIG.database.connection {
                DatabaseEngine::Mysql => {
                    let con_string = format!(
                        "mysql://{}:{}@{}:{}/{}",
                        CONFIG.database.username,
                        CONFIG.database.password,
                        CONFIG.database.host,
                        CONFIG.database.port,
                        CONFIG.database.database,
                    );

                    let pool = MySqlPool::connect(&con_string).await.map_err(|e| {
                        logger::error(&format!("MySQL connection error: {}", e));
                        e
                    })?;

                    (Box::new(MySqlGenerator), Box::new(MySqlClient { pool }))
                }

                DatabaseEngine::Sqlite => {
                    let con_string = format!("sqlite://{}", CONFIG.database.database);

                    let pool = SqlitePool::connect(&con_string).await.map_err(|e| {
                        logger::error(&format!("SQLite connection error: {}", e));
                        e
                    })?;

                    (Box::new(SqliteGenerator), Box::new(SqliteClient { pool }))
                }
            };

        Ok(Self {
            prefix: CONFIG.database.prefix.clone(),
            generator,
            client,
            debug,
            tables: HashMap::new(),
            current: None,
        })
    }

    fn fix_table_name(&self, table_name: &str) -> String {
        format!("{}{}", self.prefix, table_name)
    }

    /// Drops a table if it exists.
    ///
    /// This method:
    /// - Normalizes the provided table_name using self.fix_table_name.
    /// - Generates a DROP TABLE IF EXISTS SQL statement via self.generator.
    /// - Executes the statement using self.client.
    ///
    /// # Behavior
    /// - Returns true if the SQL executes successfully.
    /// - Returns false on error. If self.debug is true, the error is logged into log file.
    ///
    /// # Parameters
    /// - table_name: Any type convertible to String representing the table name.
    ///
    /// # Examples
    /// ```rust
    /// use rustavel_core::db::schema::Schema;
    /// async fn run()  {
    ///     let s = Schema::new().await.unwrap();
    ///     s.drop_table_if_exists("test").await;
    /// }
    /// ```
    ///
    /// # Notes
    /// - The function performs no further validation of table_name.
    /// - Errors are swallowed (converted to false) after optional logging.
    pub async fn drop_table_if_exists(&self, table_name: impl Into<String>) -> bool {
        let sql = self
            .generator
            .drop_table_if_exists(&self.fix_table_name(&table_name.into()));
        match self.client.execute(&sql).await {
            Ok(_) => true,
            Err(e) => {
                if self.debug {
                    logger::error(&format!("{:?}", e));
                }
                false
            }
        }
    }

    /// Retrieves a list of all tables in the database.
    ///
    /// This method:
    /// - Uses the database generator to create a SQL query for listing tables.
    /// - Executes the query using the database client.
    /// - Converts the result into a vector of table names.
    ///
    /// # Behavior
    /// - Returns a `Result` containing a vector of table names on success.
    /// - Returns a `DbError` if the query fails.
    /// - If `self.debug` is true, any errors will be logged.
    ///
    /// # Returns
    /// - `Ok(Vec<String>)`: A vector of table names if the query is successful.
    /// - `Err(DbError)`: An error if the table retrieval fails.
    ///
    /// # Examples
    /// ```rust
    /// use rustavel_core::db::schema::Schema;
    /// async fn run()  {
    ///     let s = Schema::new().await.unwrap();
    ///     match s.get_tables().await {
    ///         Ok(tables) => println!("Tables: {:?}", tables),
    ///         Err(e) => eprintln!("Error retrieving tables: {:?}", e),
    ///     }
    /// }
    /// ```
    ///
    /// # Notes
    /// - The exact list of tables depends on the database system and user permissions.
    /// - Performance may vary for databases with a large number of tables.
    pub async fn get_tables(&self) -> Result<Vec<String>, DbError> {
        match self
            .client
            .fetch_strings(&self.generator.get_tables())
            .await
        {
            Ok(tables) => Ok(tables),
            Err(e) => {
                if self.debug {
                    logger::error(&format!("{:?}", e));
                }
                Err(e)
            }
        }
    }
    /// Attempts to retrieve a list of tables in the database, returning an empty vector on failure.
    ///
    /// This method:
    /// - Uses the database generator to create a SQL query for listing tables.
    /// - Executes the query using the database client.
    /// - Returns the list of tables or an empty vector if the query fails.
    ///
    /// # Behavior
    /// - Returns a vector of table names if the query is successful.
    /// - Returns an empty vector if the query fails.
    /// - If `self.debug` is true, any errors will be logged.
    ///
    /// # Returns
    /// - `Vec<String>`: A list of table names, or an empty vector if retrieval fails.
    ///
    /// # Examples
    /// ```
    ///  use rustavel_core::db::schema::Schema;
    ///
    /// async fn run()  {
    ///     let s = Schema::new().await.unwrap();
    ///     let tables = s.try_get_tables().await;
    ///     println!("{}", tables.len());
    /// }
    /// ```
    ///
    /// # Notes
    /// - Unlike `get_tables()`, this method silently handles errors.
    /// - Useful in scenarios where table retrieval is optional or non-critical.
    /// - An empty result does not necessarily indicate a serious database problem.
    pub async fn try_get_tables(&self) -> Vec<String> {
        match self
            .client
            .fetch_strings(&self.generator.get_tables())
            .await
        {
            Ok(tables) => tables,
            Err(e) => {
                if self.debug {
                    logger::error(&format!("{:?}", e));
                }
                Vec::new()
            }
        }
    }

    /// Retrieves the list of columns for a specified database table.
    ///
    /// This method:
    /// - Generates a SQL query to list columns for the given table name.
    /// - Executes the query using the database client.
    /// - Returns the column names or an error if the query fails.
    ///
    /// # Behavior
    /// - Returns a `Result` containing a vector of column names on success.
    /// - Returns a `DbError` if the query fails.
    /// - If `self.debug` is true, any errors will be logged.
    ///
    /// # Parameters
    /// - `table_name`: The name of the table to retrieve columns from.
    ///
    /// # Returns
    /// - `Ok(Vec<String>)`: A vector of column names if successful.
    /// - `Err(DbError)`: An error if column retrieval fails.
    ///
    /// # Examples
    /// ```rust
    /// use rustavel_core::db::schema::Schema;
    /// async fn run()  {
    ///     let s = Schema::new().await.unwrap();
    ///     match s.get_column_listing("users").await {
    ///         Ok(columns) => println!("Columns: {:?}", columns),
    ///         Err(e) => eprintln!("Error retrieving columns: {:?}", e),
    ///     }
    /// }
    /// ```
    ///
    /// # Notes
    /// - Returns an empty vector if the table exists but has no columns or not exists at all.
    /// - The exact columns depend on the database system and table structure.
    pub async fn get_column_listing(
        &self,
        table_name: impl Into<String>,
    ) -> Result<Vec<String>, DbError> {
        match self
            .client
            .fetch_strings(
                &self
                    .generator
                    .get_column_listing(&self.fix_table_name(&table_name.into())),
            )
            .await
        {
            // note if table not found we don't have error just empty vector
            Ok(tables) => Ok(tables),
            Err(e) => {
                if self.debug {
                    logger::error(&format!("{:?}", e));
                }
                Err(e)
            }
        }
    }

    /// Attempts to retrieve the list of columns for a specified database table.
    ///
    /// This method:
    /// - Generates a SQL query to list columns for the given table name.
    /// - Executes the query using the database client.
    /// - Returns the column names or an empty vector on failure.
    ///
    /// # Behavior
    /// - Returns a vector of column names if the query is successful.
    /// - Returns an empty vector if the query fails.
    /// - If `self.debug` is true, any errors will be logged.
    ///
    /// # Parameters
    /// - `table_name`: The name of the table to retrieve columns from.
    ///
    /// # Returns
    /// - `Vec<String>`: A list of column names, or an empty vector if retrieval fails.
    ///
    /// # Examples
    /// ```rust
    /// use rustavel_core::db::schema::Schema;
    /// async fn run()  {
    ///     let s = Schema::new().await.unwrap();
    ///     let columns = s.try_get_column_listing("users").await;
    ///     println!("Columns found: {}", columns.len());
    /// }
    /// ```
    ///
    /// # Notes
    /// - Silently handles errors, making it suitable for non-critical column retrieval.
    /// - An empty result does not necessarily indicate a serious database problem.
    pub async fn try_get_column_listing(&self, table_name: impl Into<String>) -> Vec<String> {
        match self
            .client
            .fetch_strings(
                &self
                    .generator
                    .get_column_listing(&self.fix_table_name(&table_name.into())),
            )
            .await
        {
            Ok(tables) => tables,
            Err(e) => {
                if self.debug {
                    logger::error(&format!("{:?}", e));
                }
                Vec::new()
            }
        }
    }

    /// Retrieves the list of views in the database.
    ///
    /// This method:
    /// - Uses the database generator to create a SQL query for listing views.
    /// - Executes the query using the database client.
    /// - Returns the list of view names or an error if the query fails.
    ///
    /// # Behavior
    /// - Returns a `Result` containing a vector of view names on success.
    /// - Returns a `DbError` if the query fails.
    /// - If `self.debug` is true, any errors will be logged.
    ///
    /// # Returns
    /// - `Ok(Vec<String>)`: A vector of view names if the query is successful.
    /// - `Err(DbError)`: An error if view retrieval fails.
    ///
    /// # Examples
    /// ```rust
    /// use rustavel_core::db::schema::Schema;
    ///
    /// async fn run()  {
    ///     let s = Schema::new().await.unwrap();
    ///     match s.get_views().await {
    ///         Ok(views) => println!("Views: {:?}", views),
    ///         Err(e) => eprintln!("Error retrieving views: {:?}", e),
    ///     }
    /// }
    /// ```
    ///
    /// # Notes
    /// - The list of views depends on the database system and user permissions.
    /// - Performance may vary for databases with a large number of views.
    pub async fn get_views(&self) -> Result<Vec<String>, DbError> {
        match self.client.fetch_strings(&self.generator.get_views()).await {
            Ok(tables) => Ok(tables),
            Err(e) => {
                if self.debug {
                    logger::error(&format!("{:?}", e));
                }
                Err(e)
            }
        }
    }
    /// Attempts to retrieve the list of views in the database.
    ///
    /// This method:
    /// - Uses the database generator to create a SQL query for listing views.
    /// - Executes the query using the database client.
    /// - Returns the list of view names or an empty vector if the query fails.
    ///
    /// # Behavior
    /// - Returns a vector of view names if the query is successful.
    /// - Returns an empty vector if the query fails.
    /// - If `self.debug` is true, any errors will be logged.
    ///
    /// # Returns
    /// - `Vec<String>`: A list of view names, or an empty vector if retrieval fails.
    ///
    /// # Examples
    /// ```rust
    /// use rustavel_core::db::schema::Schema;
    ///     async fn run()  {
    ///     let s = Schema::new().await.unwrap();
    ///     let views = s.try_get_views().await;
    ///     println!("Views found: {}", views.len());
    /// }
    /// ```
    ///
    /// # Notes
    /// - Unlike `get_views()`, this method silently handles errors.
    /// - Useful in scenarios where view retrieval is optional or non-critical.
    /// - An empty result does not necessarily indicate a serious database problem.
    pub async fn try_get_views(&self) -> Vec<String> {
        match self.client.fetch_strings(&self.generator.get_views()).await {
            Ok(tables) => tables,
            Err(e) => {
                if self.debug {
                    logger::error(&format!("{:?}", e));
                }
                Vec::new()
            }
        }
    }

    /// WIP : may use on postgres db
    pub fn get_current_schema_name(&self) -> &str {
        &CONFIG.database.database
    }

    /// Retrieves the list of foreign keys for a specific table in the database.
    ///
    /// This method:
    /// - Uses the database generator to create a SQL query for listing foreign keys.
    /// - Fixes the table name to ensure proper formatting.
    /// - Executes the query using the database client.
    /// - Returns the list of foreign key names or an error if the query fails.
    ///
    /// # Behavior
    /// - Returns a `Result` containing a vector of foreign key names on success.
    /// - Returns a `DbError` if the query fails.
    /// - If `self.debug` is true, any errors will be logged.
    ///
    /// # Parameters
    /// - `table_name`: The name of the table to retrieve foreign keys for.
    ///
    /// # Returns
    /// - `Ok(Vec<String>)`: A vector of foreign key names if the query is successful.
    /// - `Err(DbError)`: An error if foreign key retrieval fails.
    ///
    /// # Examples
    /// ```rust
    /// use rustavel_core::db::schema::Schema;
    ///
    /// async fn run()  {
    ///     let s = Schema::new().await.unwrap();
    ///     match s.get_foreign_keys("users").await {
    ///         Ok(foreign_keys) => println!("Foreign Keys: {:?}", foreign_keys),
    ///         Err(e) => eprintln!("Error retrieving foreign keys: {:?}", e),
    ///     }
    /// }
    /// ```
    ///
    /// # Notes
    /// - The list of foreign keys depends on the database system and table structure.
    /// - Performance may vary for tables with a large number of foreign keys.
    pub async fn get_foreign_keys(
        &self,
        table_name: impl Into<String>,
    ) -> Result<Vec<String>, DbError> {
        match self
            .client
            .fetch_strings(
                &self
                    .generator
                    .get_foreign_keys(&self.fix_table_name(&table_name.into())),
            )
            .await
        {
            Ok(tables) => Ok(tables),
            Err(e) => {
                if self.debug {
                    logger::error(&format!("{:?}", e));
                }
                Err(e)
            }
        }
    }

    /// Attempts to retrieve the list of foreign keys for a specific table in the database.
    ///
    /// This method:
    /// - Uses the database generator to create a SQL query for listing foreign keys.
    /// - Fixes the table name to ensure proper formatting.
    /// - Executes the query using the database client.
    /// - Returns the list of foreign key names or an empty vector if the query fails.
    ///
    /// # Behavior
    /// - Returns a vector of foreign key names if the query is successful.
    /// - Returns an empty vector if the query fails.
    /// - If `self.debug` is true, any errors will be logged.
    ///
    /// # Parameters
    /// - `table_name`: The name of the table to retrieve foreign keys for.
    ///
    /// # Returns
    /// - `Vec<String>`: A list of foreign key names, or an empty vector if retrieval fails.
    ///
    /// # Examples
    /// ```rust
    /// use rustavel_core::db::schema::Schema;
    /// async fn run()  {
    ///     let s = Schema::new().await.unwrap();
    ///     let foreign_keys = s.try_get_foreign_keys("users").await;
    ///     println!("Foreign keys found: {}", foreign_keys.len());
    /// }
    /// ```
    ///
    /// # Notes
    /// - Unlike `get_foreign_keys()`, this method silently handles errors.
    /// - Useful in scenarios where foreign key retrieval is optional or non-critical.
    /// - An empty result does not necessarily indicate a serious database problem.
    pub async fn try_get_foreign_keys(&self, table_name: impl Into<String>) -> Vec<String> {
        match self
            .client
            .fetch_strings(
                &self
                    .generator
                    .get_foreign_keys(&self.fix_table_name(&table_name.into())),
            )
            .await
        {
            Ok(tables) => tables,
            Err(e) => {
                if self.debug {
                    logger::error(&format!("{:?}", e));
                }
                Vec::new()
            }
        }
    }

    /// Drops a specific table from the database.
    ///
    /// This method:
    /// - Fixes the table name to ensure proper formatting.
    /// - Uses the database generator to create a SQL DROP TABLE command.
    /// - Executes the drop command using the database client.
    ///
    /// # Behavior
    /// - Attempts to remove the specified table from the database.
    /// - Returns success if the table is dropped or does not exist.
    /// - Logs any errors if debug mode is enabled.
    ///
    /// # Parameters
    /// - `table_name`: The name of the table to be dropped.
    ///
    /// # Returns
    /// - `Ok(())`: Successful table drop operation.
    /// - `Err(DbError)`: Error encountered during table drop.
    ///
    /// # Examples
    /// ```rust
    /// use rustavel_core::db::schema::Schema;
    /// async fn run()  {
    ///     let s = Schema::new().await.unwrap();
    ///     match s.drop_table("users").await {
    ///         Ok(_) => println!("Table dropped successfully"),
    ///         Err(e) => eprintln!("Error dropping table: {:?}", e),
    ///     }
    /// }
    /// ```
    ///
    /// # Notes
    /// - Be cautious when dropping tables as this action is irreversible.
    /// - Requires appropriate database permissions.
    pub async fn drop_table(&self, table_name: impl Into<String>) -> Result<(), DbError> {
        self.client
            .execute(
                &self
                    .generator
                    .drop_table(&self.fix_table_name(&table_name.into())),
            )
            .await
            .map_err(|e| {
                if self.debug {
                    logger::error(&format!("{:?}", e));
                }
                e
            })
    }
    /// Drops all tables in the database.
    ///
    /// This method:
    /// - Uses the database generator to create a SQL command to drop all tables.
    /// - Executes the drop all tables command using the database client.
    ///
    /// # Behavior
    /// - Attempts to remove all tables from the database.
    /// - Returns success if all tables are dropped or no tables exist.
    /// - Logs any errors if debug mode is enabled.
    ///
    /// # Returns
    /// - `Ok(())`: Successful drop of all tables.
    /// - `Err(DbError)`: Error encountered during table drop operation.
    ///
    /// # Examples
    /// ```rust
    /// use rustavel_core::db::schema::Schema;
    /// async fn run()  {
    ///     let s = Schema::new().await.unwrap();
    ///     match s.drop_all_tables().await {
    ///         Ok(_) => println!("All tables dropped successfully"),
    ///         Err(e) => eprintln!("Error dropping all tables: {:?}", e),
    ///     }
    /// }
    /// ```
    ///
    /// # Notes
    /// - Extremely destructive operation - use with extreme caution.
    /// - Requires highest level of database permissions.
    /// - Permanently removes all data in all tables.
    pub async fn drop_all_tables(&self) -> Result<(), DbError> {
        self.client
            .execute(&self.generator.drop_all_tables())
            .await
            .map_err(|e| {
                if self.debug {
                    logger::error(&format!("{:?}", e));
                }
                e
            })
    }

    /// Drops all views in the database.
    ///
    /// This method:
    /// - Uses the database generator to create a SQL command to drop all views.
    /// - Executes the drop all views command using the database client.
    ///
    /// # Behavior
    /// - Attempts to remove all views from the database.
    /// - Returns success if all views are dropped or no views exist.
    /// - Logs any errors if debug mode is enabled.
    ///
    /// # Returns
    /// - `Ok(())`: Successful drop of all views.
    /// - `Err(DbError)`: Error encountered during view drop operation.
    ///
    /// # Examples
    /// ```rust
    /// use rustavel_core::db::schema::Schema;
    ///
    /// async fn run()  {
    ///     let s = Schema::new().await.unwrap();
    ///     match s.drop_all_views().await {
    ///         Ok(_) => println!("All views dropped successfully"),
    ///         Err(e) => eprintln!("Error dropping all views: {:?}", e),
    ///     }
    /// }
    /// ```
    ///
    /// # Notes
    /// - Permanently removes all database views.
    /// - Requires appropriate database permissions.
    pub async fn drop_all_views(&self) -> Result<(), DbError> {
        self.client
            .execute(&self.generator.drop_all_views())
            .await
            .map_err(|e| {
                if self.debug {
                    logger::error(&format!("{:?}", e));
                }
                e
            })
    }

    /// Checks if a specific column exists in a given table.
    ///
    /// This method:
    /// - Fixes the table name to ensure proper formatting.
    /// - Uses the database generator to create a SQL query to check column existence.
    /// - Executes the query using the database client.
    ///
    /// # Behavior
    /// - Determines whether the specified column exists in the table.
    /// - Returns a boolean indicating column presence.
    /// - Logs any errors if debug mode is enabled.
    ///
    /// # Parameters
    /// - `table_name`: The name of the table to check.
    /// - `column_name`: The name of the column to search for.
    ///
    /// # Returns
    /// - `Ok(bool)`:
    ///   - `true` if the column exists in the table.
    ///   - `false` if the column does not exist.
    /// - `Err(DbError)`: Error encountered during column existence check.
    ///
    /// # Examples
    /// ```rust
    /// use rustavel_core::db::schema::Schema;
    /// async fn run()  {
    ///     let s = Schema::new().await.unwrap();
    ///     match s.has_column("users", "email").await {
    ///         Ok(exists) => println!("Column exists: {}", exists),
    ///         Err(e) => eprintln!("Error checking column: {:?}", e),
    ///     }
    /// }
    /// ```
    ///
    /// # Notes
    /// - Useful for schema introspection and validation.
    /// - Performance may vary depending on the database system.
    pub async fn has_column(
        &self,
        table_name: impl Into<String>,
        column_name: impl Into<String>,
    ) -> Result<bool, DbError> {
        let sql = self.generator.has_column(
            &self.fix_table_name(&table_name.into()),
            &column_name.into(),
        );
        match self.client.fetch_strings(&sql).await {
            Ok(handler) => Ok(!handler.is_empty()),
            Err(e) => {
                if self.debug {
                    logger::error(&format!("{:?}", e));
                }
                Err(e)
            }
        }
    }

    /// Checks if a specific table exists in the database.
    ///
    /// This method:
    /// - Fixes the table name to ensure proper formatting.
    /// - Uses the database generator to create a SQL query to check table existence.
    /// - Executes the query using the database client.
    ///
    /// # Behavior
    /// - Determines whether the specified table exists in the database.
    /// - Returns a boolean indicating table presence.
    /// - Logs any errors if debug mode is enabled.
    ///
    /// # Parameters
    /// - `table_name`: The name of the table to check.
    ///
    /// # Returns
    /// - `Ok(bool)`:
    ///   - `true` if the table exists in the database.
    ///   - `false` if the table does not exist.
    /// - `Err(DbError)`: Error encountered during table existence check.
    ///
    /// # Examples
    /// ```rust
    /// use rustavel_core::db::schema::Schema;
    /// async fn run()  {
    ///     let s = Schema::new().await.unwrap();
    ///     match s.has_table("users").await {
    ///         Ok(exists) => println!("Table exists: {}", exists),
    ///         Err(e) => eprintln!("Error checking table: {:?}", e),
    ///     }
    /// }
    /// ```
    ///
    /// # Notes
    /// - Useful for schema introspection and validation.
    /// - Performance may vary depending on the database system.
    pub async fn has_table(&self, table_name: impl Into<String>) -> Result<bool, DbError> {
        let sql = self
            .generator
            .has_table(&self.fix_table_name(&table_name.into()));
        match self.client.fetch_strings(&sql).await {
            Ok(handler) => Ok(!handler.is_empty()),
            Err(e) => {
                if self.debug {
                    logger::error(&format!("{:?}", e));
                }
                Err(e)
            }
        }
    }

    /// Checks if a specific view exists in the database.
    ///
    /// This method:
    /// - Fixes the view name to ensure proper formatting.
    /// - Uses the database generator to create a SQL query to check view existence.
    /// - Executes the query using the database client.
    ///
    /// # Behavior
    /// - Determines whether the specified view exists in the database.
    /// - Returns a boolean indicating view presence.
    /// - Logs any errors if debug mode is enabled.
    ///
    /// # Parameters
    /// - `table_name`: The name of the view to check.
    ///
    /// # Returns
    /// - `Ok(bool)`:
    ///   - `true` if the view exists in the database.
    ///   - `false` if the view does not exist.
    /// - `Err(DbError)`: Error encountered during view existence check.
    ///
    /// # Examples
    /// ```rust
    /// use rustavel_core::db::schema::Schema;
    /// async fn run()  {
    ///     let s = Schema::new().await.unwrap();
    ///     match s.has_view("user_summary").await {
    ///         Ok(exists) => println!("View exists: {}", exists),
    ///         Err(e) => eprintln!("Error checking view: {:?}", e),
    ///     }
    /// }
    /// ```
    ///
    /// # Notes
    /// - Useful for schema introspection and validation.
    /// - Performance may vary depending on the database system.
    pub async fn has_view(&self, table_name: impl Into<String>) -> Result<bool, DbError> {
        let sql = self
            .generator
            .has_view(&self.fix_table_name(&table_name.into()));
        match self.client.fetch_strings(&sql).await {
            Ok(handler) => Ok(!handler.is_empty()),
            Err(e) => {
                if self.debug {
                    logger::error(&format!("{:?}", e));
                }
                Err(e)
            }
        }
    }

    /// Checks if a specific index exists on a table for given columns.
    ///
    /// This method:
    /// - Fixes the table name to ensure proper formatting.
    /// - Uses the database generator to create a SQL query to check index existence.
    /// - Executes the query using the database client.
    ///
    /// # Behavior
    /// - Determines whether an index exists on the specified table for given columns.
    /// - Returns a boolean indicating index presence.
    /// - Logs any errors if debug mode is enabled.
    ///
    /// # Parameters
    /// - `table_name`: The name of the table to check for the index.
    /// - `columns_name`: A vector of column names that compose the index.
    ///
    /// # Returns
    /// - `Ok(bool)`:
    ///   - `true` if the index exists on the specified columns.
    ///   - `false` if the index does not exist.
    /// - `Err(DbError)`: Error encountered during index existence check.
    ///
    /// # Examples
    /// ```rust
    /// use rustavel_core::db::schema::Schema;
    ///
    /// async fn run()  {
    ///     let s = Schema::new().await.unwrap();
    ///     match s.has_index("users", vec!["email", "username"]).await {
    ///         Ok(exists) => println!("Index exists: {}", exists),
    ///         Err(e) => eprintln!("Error checking index: {:?}", e),
    ///     }
    /// }
    ///
    pub async fn has_index(
        &self,
        table_name: impl Into<String>,
        columns_name: Vec<&str>,
    ) -> Result<bool, DbError> {
        let sql = self
            .generator
            .has_index(&self.fix_table_name(&table_name.into()), columns_name);
        match self.client.fetch_strings(&sql).await {
            Ok(handler) => Ok(!handler.is_empty()),
            Err(e) => {
                if self.debug {
                    logger::error(&format!("{:?}", e));
                }
                Err(e)
            }
        }
    }

    /// Creates a new database.
    ///
    /// # Behavior
    /// - Attempts to create a new database using the database generator.
    /// - Returns a boolean indicating success or failure.
    /// - Logs any errors if debug mode is enabled.
    ///
    /// # Parameters
    /// - `db_name`: The name of the database to create.
    ///
    /// # Returns
    /// - `true` if database creation was successful.
    /// - `false` if database creation failed.
    ///
    /// # Examples
    /// ```rust
    /// use rustavel_core::db::schema::Schema;
    ///
    /// async fn run() {
    ///     let s = Schema::new().await.unwrap();
    ///     let created = s.create_database("my_new_database").await;
    ///     println!("Database created: {}", created);
    /// }
    /// ```
    ///
    /// # Notes
    /// - Not supported in SQLite, which uses file-based databases.
    /// - Behavior varies across different database systems.
    /// - Requires appropriate database permissions.
    pub async fn create_database(&self, db_name: impl Into<String>) -> bool {
        let sql = self.generator.create_database(&db_name.into());
        match self.client.execute(&sql).await {
            Ok(_) => true,
            Err(e) => {
                if self.debug {
                    logger::error(&format!("{:?}", e));
                }
                false
            }
        }
    }

    /// Drops a database if it exists.
    ///
    /// # Behavior
    /// - Attempts to drop the specified database using the database generator.
    /// - Returns a boolean indicating success or failure.
    /// - Logs any errors if debug mode is enabled.
    ///
    /// # Parameters
    /// - `db_name`: The name of the database to drop.
    ///
    /// # Returns
    /// - `true` if database drop was successful or database did not exist.
    /// - `false` if database drop failed.
    ///
    /// # Examples
    /// ```rust
    /// use rustavel_core::db::schema::Schema;;
    /// async fn run()  {
    ///     let s = Schema::new().await.unwrap();
    ///     let dropped = s.drop_database_if_exists("old_database").await;
    ///     println!("Database dropped: {}", dropped);
    /// }
    /// ```
    ///
    /// # Notes
    /// - Not supported in SQLite, which uses file-based databases.
    /// - Behavior varies across different database systems.
    /// - Requires highest level of database permissions.
    /// - Permanently removes the entire database and all its contents.
    pub async fn drop_database_if_exists(&self, db_name: impl Into<String>) -> bool {
        match self
            .client
            .execute(&self.generator.drop_database_if_exists(&db_name.into()))
            .await
        {
            Ok(_) => true,
            Err(e) => {
                if self.debug {
                    logger::error(&format!("{:?}", e));
                }
                false
            }
        }
    }

    /// Disables foreign key constraints for the specified database.
    ///
    /// # Behavior
    /// - Attempts to disable foreign key constraints using the database generator.
    /// - Returns a boolean indicating success or failure.
    /// - Logs any errors if debug mode is enabled.
    ///
    /// # Parameters
    /// - `db_name`: The name of the database to disable constraints for.
    ///
    /// # Returns
    /// - `true` if disabling foreign key constraints was successful.
    /// - `false` if the operation failed.
    ///
    /// # Examples
    /// ```rust
    /// use rustavel_core::db::schema::Schema;
    /// async fn run()  {
    ///     let s = Schema::new().await.unwrap();
    ///     let disabled = s.disable_foreign_key_constraints("my_database").await;
    ///     println!("Foreign key constraints disabled: {}", disabled);
    /// }
    /// ```
    ///
    /// # Notes
    /// - Not supported in all database systems.
    /// - Useful for bulk data operations or migrations where foreign key checks need to be temporarily suspended.
    /// - Requires appropriate database permissions.
    pub async fn disable_foreign_key_constraints(&self, db_name: impl Into<String>) -> bool {
        match self
            .client
            .execute(
                &self
                    .generator
                    .disable_foreign_key_constraints(&db_name.into()),
            )
            .await
        {
            Ok(_) => true,
            Err(e) => {
                if self.debug {
                    logger::error(&format!("{:?}", e));
                }
                false
            }
        }
    }

    /// Enables foreign key constraints for the specified database.
    ///
    /// # Behavior
    /// - Attempts to enable foreign key constraints using the database generator.
    /// - Returns a boolean indicating success or failure.
    /// - Logs any errors if debug mode is enabled.
    ///
    /// # Parameters
    /// - `db_name`: The name of the database to enable constraints for.
    ///
    /// # Returns
    /// - `true` if enabling foreign key constraints was successful.
    /// - `false` if the operation failed.
    ///
    /// # Examples
    /// ```rust
    /// use rustavel_core::db::schema::Schema;
    /// async fn run()  {
    ///     let s = Schema::new().await.unwrap();
    ///     let enabled = s.enable_foreign_key_constraints("my_database").await;
    ///     println!("Foreign key constraints enabled: {}", enabled);
    /// }
    /// ```
    ///
    /// # Notes
    /// - Not supported in all database systems.
    /// - Typically used after performing operations that required foreign key constraints to be disabled.
    /// - Requires appropriate database permissions.
    pub async fn enable_foreign_key_constraints(&self, db_name: impl Into<String>) -> bool {
        match self
            .client
            .execute(
                &self
                    .generator
                    .enable_foreign_key_constraints(&db_name.into()),
            )
            .await
        {
            Ok(_) => true,
            Err(e) => {
                if self.debug {
                    logger::error(&format!("{:?}", e));
                }
                false
            }
        }
    }

    /// Renames a table in the database.
    ///
    /// This method:
    /// - Uses the database generator to create a SQL RENAME TABLE command.
    /// - Executes the rename operation using the database client.
    ///
    /// # Behavior
    /// - Attempts to rename a table from the old name to the new name.
    /// - Logs any errors if debug mode is enabled.
    /// - Returns success if the table is successfully renamed.
    ///
    /// # Parameters
    /// - `old_table_name`: The current name of the table.
    /// - `new_table_name`: The desired new name for the table.
    ///
    /// # Returns
    /// - `Ok(())`: Successful table renaming.
    /// - `Err(DbError)`: Error encountered during table renaming.
    ///
    /// # Examples
    /// ```rust
    /// use rustavel_core::db::schema::Schema;
    /// async fn run()  {
    ///     let s = Schema::new().await.unwrap();
    ///     match s.rename("old_users", "new_users").await {
    ///         Ok(_) => println!("Table renamed successfully"),
    ///         Err(e) => eprintln!("Error renaming table: {:?}", e),
    ///     }
    /// }
    /// ```
    ///
    /// # Notes
    /// - Behavior may vary across different database systems.
    /// - Requires appropriate database permissions.
    /// - Renaming a table can potentially break existing references or queries.
    pub async fn rename(
        &self,
        old_table_name: impl Into<String>,
        new_table_name: impl Into<String>,
    ) -> Result<(), DbError> {
        let sql = self
            .generator
            .rename(&old_table_name.into(), &new_table_name.into());
        match self.client.execute(&sql).await {
            Ok(_) => Ok(()),
            Err(e) => {
                if self.debug {
                    logger::error(&format!("{:?}", e));
                }
                Err(e)
            }
        }
    }

    /// Renames all tables by replacing the current prefix with a new prefix.
    ///
    /// This method:
    /// - Retrieves the list of all tables in the database.
    /// - Renames each table by replacing the existing prefix with the new prefix.
    /// - Updates the internal prefix value after successful renaming.
    ///
    /// # Behavior
    /// - Attempts to rename all tables with the new prefix.
    /// - Logs a warning about updating the configuration file.
    /// - Returns success if all table renames are successful.
    /// - Returns an error if table retrieval or renaming fails.
    ///
    /// # Parameters
    /// - `new_prefix`: The new prefix to be applied to table names.
    ///
    /// # Returns
    /// - `Ok(())`: Successfully renamed all tables and updated prefix.
    /// - `Err(DbError)`: Error encountered during table renaming process.
    ///
    /// # Examples
    /// ```rust
    /// use rustavel_core::db::schema::Schema;
    /// async fn run()  {
    ///     let mut s = Schema::new().await.unwrap();
    ///     match s.rename_prefix("new_").await {
    ///         Ok(_) => println!("All tables renamed successfully"),
    ///         Err(e) => eprintln!("Error renaming tables: {:?}", e),
    ///     }
    /// }
    /// ```
    ///
    /// # Notes
    /// - Requires manually updating the prefix in the configuration file too.
    /// - Potentially destructive operation - use with caution.
    /// - Affects all tables in the database with the current prefix.
    pub async fn rename_prefix(&mut self, new_prefix: &str) -> Result<(), DbError> {
        match self.get_tables().await {
            Ok(tables) => {
                for table in tables {
                    self.rename(table.clone(), table.replace(&self.prefix, new_prefix))
                        .await?
                }
                self.prefix = new_prefix.to_string();
                logger::warn("After change prefix you need to fix your config prefix too `.env`");
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub fn create<F>(&mut self, table_name: impl Into<String>, f: F) -> &mut Self
    where
        F: FnOnce(&mut Table),
    {
        let name = table_name.into();
        let mut table = Table::new(&format!("{}{}", CONFIG.database.prefix, name));
        table.action = TableAction::Create;
        f(&mut table);


        // check if the table already exists
        if let Some(tbl) = self.tables.get_mut(&name) {

            // update table state only if the existing table is the same
            for column in &table.columns {
                tbl.columns.push(column.clone());
            }
        } else {
            // insert the new table if it doesn't exist
            self.tables.insert(name.clone(), table.clone());
        }

        self.current = Some(table);
        // println!("{:?}",self.tables.keys().cloned().collect::<Vec<_>>());

        self
        // eprintln!("struct {}",&table.to_struct());
        // let mut body = vec![];
        // let mut foot = vec![];
        // let mut post = vec![];
        // for column in table.columns {
        //     let (b, f, p) = self.generator.column(&column, &name, &table.action);
        //     body.push(b);
        //     if !f.is_empty() {
        //         foot.push(f);
        //     }
        //     if !p.is_empty() {
        //         post.push(p);
        //     }
        // }
        // for key in table.foreign_keys {
        //     let str = self.generator.foreign_key(&key, &name, &table.action);
        //
        //     if !str.is_empty() {
        //         foot.push(str);
        //     }
        // }
        // body.append(&mut foot);
        // let sql = Str::implode(",\n", body);
        // let sql = self.generator.table_sql(&name, &sql, &Str::implode(";\n", post), &table.action);
        // logger::info(&format!("Just4debug develop core: \n {}", sql));
        // match self.client.execute(&sql).await {
        //     Ok(_) => {
        //         logger::success(&format!("Created table  : \n {}", name));
        //     },
        //     Err(e) => {
        //         logger::error(&format!("{:?}", e));
        //     }
        // }


    }

    pub fn table<F>(&mut self, table_name: impl Into<String>, f: F) -> &mut Self
    where
        F: FnOnce(&mut Table),
    {
        let name = table_name.into();
        let mut table = Table::new(&format!("{}{}", CONFIG.database.prefix, name));
        table.action = TableAction::Alter;

        f(&mut table);

        // println!("{:?}",self.tables.keys().cloned().collect::<Vec<_>>());
        // check if the table already exists
        if let Some(tbl) = self.tables.get_mut(&name) {

            // update table state only if the existing table is the same
            for column in &table.columns {
                tbl.columns.push(column.clone());
            }
            // dbg!(&tbl.columns);

        } else {
            // insert the new table if it doesn't exist
            self.tables.insert(name.clone(), table.clone());
        }


        self.current = Some(table);

        self


    }

    /// execute last migration generated by schema
    pub async fn execute_migration(&self, final_name: &str ,duration: &Instant) -> Result<(),DbError> {

        if let Some(table) = &self.current {
            let mut body = vec![];
            let mut foot = vec![];
            let mut post = vec![];
            for column in &table.columns {
                let (b, f, p) = self.generator.column(&column, &table.name, &table.action);
                body.push(b);
                if !f.is_empty() {
                    foot.push(f);
                }
                if !p.is_empty() {
                    post.push(p);
                }
            }
            for column in &table.drop_columns {
                body.push(self.generator.drop_column(&column));
            }
            for key in &table.foreign_keys {
                let str = self.generator.foreign_key(&key, &table.name, &table.action);

                if !str.is_empty() {
                    foot.push(str);
                }
            }
            body.append(&mut foot);
            let sql = Str::implode(",\n", body);
            let sql = self.generator.table_sql(&table.name, &sql, &Str::implode(";\n", post), &table.action);
            // logger::info(&format!("Just4debug develop core: \n {}", sql));
            match self.client.execute(&sql).await {
                Ok(_) => {
                    // logger::success(&format!("Updated table  : \n {}", &table.name));
                    operation(
                        &format!("migration executed: {}", final_name),
                        duration.elapsed(),
                        Status::Done,
                    );
                    Ok(())
                },
                Err(e) => {
                    // logger::error(&format!("{:?}", e));
                    operation(
                        &format!("migration executed: {}", final_name),
                        duration.elapsed(),
                        Status::Failed,
                    );
                    Err(e)
                }
            }
        }else {
            Err(DbError::InvalidTable)
        }

    }
}
