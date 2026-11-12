// NOTE: use the same `use ...` imports as the MySQL generator file
// (SqlGenerator, Column, ColumnDataType, ColumnOption, DefaultValue, ForeignKey, TableAction, logger).

use crate::db::table::{Column, ColumnDataType, ColumnOption, DefaultValue, ForeignKey, TableAction};
use crate::logger;
use crate::sql::generator::SqlGenerator;

#[derive(Debug)]
#[allow(dead_code)]
pub struct PostgresGenerator;

/// Quotes an identifier for PostgreSQL: name -> "name" (embedded `"` is doubled).
fn ident(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

/// Escapes a value placed inside '...' in PostgreSQL.
/// `standard_conforming_strings` is ON by default, so a backslash is an ordinary
/// character here (unlike MySQL) and only the single quote must be doubled.
fn quote_str(value: &str) -> String {
    value.replace('\'', "''")
}

#[allow(dead_code)]
impl PostgresGenerator {
    #[inline]
    fn db(&self) -> &'static str {
        "current_schema()"
    }
}

impl SqlGenerator for PostgresGenerator {
    fn drop_table_if_exists(&self, table_name: &str) -> String {
        format!("DROP TABLE IF EXISTS {};", ident(table_name))
    }

    fn get_tables(&self) -> String {
        "
        SELECT table_name
        FROM information_schema.tables
        WHERE table_schema = current_schema()
          AND table_type = 'BASE TABLE'
        "
            .to_string()
    }

    fn get_views(&self) -> String {
        "
        SELECT table_name
        FROM information_schema.views
        WHERE table_schema = current_schema()
        "
            .to_string()
    }

    fn get_column_listing(&self, table_name: &str) -> String {
        format!(
            "
        SELECT column_name
        FROM information_schema.columns
        WHERE table_schema = current_schema() AND table_name = '{}'
        ORDER BY ordinal_position;
        ",
            quote_str(table_name)
        )
    }

    fn get_foreign_keys(&self, table_name: &str) -> String {
        format!(
            "
            SELECT constraint_name
            FROM information_schema.table_constraints
            WHERE table_schema = current_schema()
              AND table_name = '{}'
              AND constraint_type = 'FOREIGN KEY'
            ",
            quote_str(table_name)
        )
    }

    fn drop_table(&self, table_name: &str) -> String {
        format!("DROP TABLE {};", ident(table_name))
    }

    fn drop_view(&self, view_name: &str) -> String {
        format!("DROP VIEW {};", ident(view_name))
    }

    fn has_column(&self, table_name: &str, column_name: &str) -> String {
        format!(
            "
            SELECT '1'
            FROM information_schema.columns
            WHERE table_schema = current_schema()
              AND table_name = '{}'
              AND column_name = '{}'
            LIMIT 1
            ",
            quote_str(table_name),
            quote_str(column_name)
        )
    }

    fn has_table(&self, table_name: &str) -> String {
        format!(
            "
            SELECT '1'
            FROM information_schema.tables
            WHERE table_schema = current_schema()
              AND table_name = '{}'
            LIMIT 1
            ",
            quote_str(table_name)
        )
    }

    fn has_view(&self, table_name: &str) -> String {
        format!(
            "
            SELECT '1'
            FROM information_schema.views
            WHERE table_schema = current_schema()
              AND table_name = '{}'
            LIMIT 1
            ",
            quote_str(table_name)
        )
    }

    // PostgreSQL has no information_schema.statistics, so we use the pg_* catalogs.
    // An index matches when it contains exactly the given set of columns.
    fn has_index(&self, table_name: &str, columns_name: Vec<&str>) -> String {
        let cols = columns_name
            .iter()
            .map(|c| format!("'{}'", quote_str(c)))
            .collect::<Vec<_>>()
            .join(",");

        let count = columns_name.len();

        format!(
            "
            SELECT '1'
            FROM pg_index ix
            JOIN pg_class t ON t.oid = ix.indrelid
            JOIN pg_class i ON i.oid = ix.indexrelid
            JOIN pg_namespace n ON n.oid = t.relnamespace
            JOIN pg_attribute a ON a.attrelid = t.oid AND a.attnum = ANY(ix.indkey)
            WHERE n.nspname = current_schema()
              AND t.relname = '{}'
              AND ix.indnatts = {}
              AND a.attname IN ({})
            GROUP BY i.oid
            HAVING COUNT(*) = {}
            LIMIT 1
            ",
            quote_str(table_name),
            count,
            cols,
            count
        )
    }

    fn create_database(&self, db_name: &str) -> String {
        format!("CREATE DATABASE {};", ident(db_name))
    }

    fn drop_database_if_exists(&self, db_name: &str) -> String {
        format!("DROP DATABASE IF EXISTS {};", ident(db_name))
    }

    // Same as Laravel's PostgresGrammar. Only affects DEFERRABLE constraints and
    // only inside the current transaction.
    fn disable_foreign_key_constraints(&self) -> String {
        "SET CONSTRAINTS ALL DEFERRED;".to_string()
    }

    fn enable_foreign_key_constraints(&self) -> String {
        "SET CONSTRAINTS ALL IMMEDIATE;".to_string()
    }

    fn rename(&self, old_table_name: &str, new_table_name: &str) -> String {
        format!(
            "ALTER TABLE {} RENAME TO {};",
            ident(old_table_name),
            ident(new_table_name)
        )
    }

    /// Returns (column_sql, footer_sql, post_sql, separator).
    ///
    /// * column_sql : column definition(s) (`ADD COLUMN ...` / `ALTER COLUMN ...` on alter)
    /// * footer_sql : constraints that may live inside CREATE TABLE / ALTER TABLE (UNIQUE)
    /// * post_sql   : statements that must run AFTER the table statement, because
    ///                PostgreSQL cannot declare indexes or comments inline
    ///                (CREATE INDEX, COMMENT ON COLUMN)
    fn column(
        &self,
        column: &Column,
        table_name: &str,
        action: &TableAction,
    ) -> (String, String, String, String) {
        let col = ident(&column.name);
        let tbl = ident(table_name);

        let nullable = if column.nullable { "NULL" } else { "NOT NULL" };

        // default expression without the DEFAULT keyword (needed for SET DEFAULT on change)
        let default_expr: Option<String> = match &column.default {
            DefaultValue::Null => Some("NULL".to_string()),
            DefaultValue::String(s) => Some(format!("'{}'", quote_str(s))),
            DefaultValue::JsonArray => Some("'[]'".to_string()),
            DefaultValue::CurrenTimestamp => Some("CURRENT_TIMESTAMP".to_string()),
            DefaultValue::Bool(b) => Some((if *b { "TRUE" } else { "FALSE" }).to_string()),
            DefaultValue::Int(i) => Some(i.to_string()),
            _ => None,
        };
        let default_sql = default_expr
            .as_ref()
            .map(|d| format!(" DEFAULT {}", d))
            .unwrap_or_default();

        let mut defs: Vec<String> = Vec::new();
        let mut footers: Vec<String> = Vec::new();
        let mut post: Vec<String> = Vec::new();

        // single-column types: (sql type, optional CHECK expression)
        let mut single: Option<(String, Option<String>)> = None;

        match column.data_type {
            ColumnDataType::DTId => {
                defs.push(format!("{} BIGSERIAL PRIMARY KEY", ident("id")));
            }
            ColumnDataType::DTBoolean => {
                single = Some(("BOOLEAN".to_string(), None));
            }
            // PostgreSQL has no TINYINT / MEDIUMINT / UNSIGNED (Laravel ignores unsigned too)
            ColumnDataType::DTTinyInteger | ColumnDataType::DTSmallInteger => {
                single = Some(("SMALLINT".to_string(), None));
            }
            ColumnDataType::DTInteger | ColumnDataType::DTMediumInteger => {
                single = Some(("INTEGER".to_string(), None));
            }
            ColumnDataType::DTBigInteger => {
                single = Some(("BIGINT".to_string(), None));
            }
            ColumnDataType::DTFloat => {
                single = Some(("REAL".to_string(), None));
            }
            ColumnDataType::DTDouble => {
                single = Some(("DOUBLE PRECISION".to_string(), None));
            }
            ColumnDataType::DTDecimal => {
                let (precision, scale) = match &column.option {
                    ColumnOption::Float((p, s)) => (i32::from(*p), i32::from(*s)),
                    _ => (20, 6),
                };
                single = Some((format!("DECIMAL({},{})", precision, scale), None));
            }
            ColumnDataType::DTString => {
                let len = match column.option {
                    ColumnOption::Length(l) => l,
                    _ => 127,
                };
                single = Some((format!("VARCHAR({})", len), None));
            }
            // PostgreSQL's TEXT has no size variants
            ColumnDataType::DTText
            | ColumnDataType::DTTinyText
            | ColumnDataType::DTMediumText
            | ColumnDataType::DTLongText => {
                single = Some(("TEXT".to_string(), None));
            }
            // Laravel's json() -> json. Swap to JSONB if you prefer indexing/operators.
            ColumnDataType::DTJson => {
                single = Some(("JSON".to_string(), None));
            }
            ColumnDataType::DTDate => {
                single = Some(("DATE".to_string(), None));
            }
            ColumnDataType::DTDateTime | ColumnDataType::DTTimestamp => {
                single = Some(("TIMESTAMP(0) WITHOUT TIME ZONE".to_string(), None));
            }
            ColumnDataType::DTTime => {
                single = Some(("TIME(0) WITHOUT TIME ZONE".to_string(), None));
            }
            ColumnDataType::DTTimestamps => {
                defs.push(format!(
                    "{} TIMESTAMP(0) WITHOUT TIME ZONE NULL",
                    ident("created_at")
                ));
                defs.push(format!(
                    "{} TIMESTAMP(0) WITHOUT TIME ZONE NULL",
                    ident("updated_at")
                ));
            }
            ColumnDataType::DTSoftDelete => {
                defs.push(format!(
                    "{} TIMESTAMP(0) WITHOUT TIME ZONE NULL",
                    ident("deleted_at")
                ));
            }
            // Laravel: enum -> varchar(255) + CHECK (col IN (...))
            ColumnDataType::DTEnum => {
                let enums = match &column.option {
                    ColumnOption::Values(items) => items
                        .iter()
                        .map(|item| format!("'{}'", quote_str(item)))
                        .collect::<Vec<_>>()
                        .join(", "),
                    _ => "''".to_string(),
                };
                single = Some((
                    "VARCHAR(255)".to_string(),
                    Some(format!("CHECK ({} IN ({}))", col, enums)),
                ));
            }
            // PostgreSQL has no SET type: emulated as TEXT[] restricted to the allowed values.
            // A string default must therefore be an array literal, e.g. '{a,b}'.
            ColumnDataType::DTSet => {
                let sets = match &column.option {
                    ColumnOption::Values(items) => items
                        .iter()
                        .map(|item| format!("'{}'", quote_str(item)))
                        .collect::<Vec<_>>()
                        .join(", "),
                    _ => "''".to_string(),
                };
                single = Some((
                    "TEXT[]".to_string(),
                    Some(format!("CHECK ({} <@ ARRAY[{}]::TEXT[])", col, sets)),
                ));
            }
            ColumnDataType::DTMorph => {
                let type_col = ident(&format!("{}_type", column.name));
                let id_col = ident(&format!("{}_id", column.name));
                defs.push(format!("{} VARCHAR(255) {}{}", type_col, nullable, default_sql));
                defs.push(format!("{} BIGINT {}{}", id_col, nullable, default_sql));
                post.push(format!(
                    "CREATE INDEX {} ON {} ({}, {});",
                    ident(&format!(
                        "{}_{}_type_{}_id_index",
                        table_name, column.name, column.name
                    )),
                    tbl,
                    type_col,
                    id_col
                ));
            }
            _ => {}
        }

        // PostgreSQL has no CHANGE COLUMN: a change is a set of ALTER COLUMN actions
        let changing =
            *action == TableAction::Alter && column.change && single.is_some();

        if let Some((sql_type, check)) = &single {
            let collate = if !column.collation.is_empty() && column.is_string_type() {
                format!(" COLLATE {}", ident(&column.collation))
            } else {
                String::new()
            };
            let check_name = ident(&format!("{}_{}_check", table_name, column.name));

            if changing {
                let mut actions = vec![format!(
                    "ALTER COLUMN {col} TYPE {sql_type}{collate} USING {col}::{sql_type}"
                )];
                actions.push(format!(
                    "ALTER COLUMN {col} {}",
                    if column.nullable { "DROP NOT NULL" } else { "SET NOT NULL" }
                ));
                match &default_expr {
                    Some(d) => actions.push(format!("ALTER COLUMN {col} SET DEFAULT {d}")),
                    None => actions.push(format!("ALTER COLUMN {col} DROP DEFAULT")),
                }
                if let Some(c) = check {
                    actions.push(format!("DROP CONSTRAINT IF EXISTS {check_name}"));
                    actions.push(format!("ADD CONSTRAINT {check_name} {c}"));
                }
                defs.push(actions.join(",\n"));
            } else {
                let check_sql = check
                    .as_ref()
                    .map(|c| format!(" CONSTRAINT {} {}", check_name, c))
                    .unwrap_or_default();
                defs.push(format!(
                    "{} {}{}{} {}{}",
                    col, sql_type, collate, check_sql, nullable, default_sql
                ));
            }

            // PostgreSQL has no inline COMMENT
            if !column.comment.is_empty() {
                post.push(format!(
                    "COMMENT ON COLUMN {}.{} IS '{}';",
                    tbl,
                    col,
                    quote_str(&column.comment)
                ));
            }
        }

        // unique can live inside the table statement; plain indexes cannot
        if column.unique {
            footers.push(format!(
                "CONSTRAINT {} UNIQUE ({})",
                ident(&format!("{}_{}_unique", table_name, column.name)),
                col
            ));
        } else if column.index {
            post.push(format!(
                "CREATE INDEX {} ON {} ({});",
                ident(&format!("{}_{}_index", table_name, column.name)),
                tbl,
                col
            ));
        }

        if *action == TableAction::Alter {
            if column.change && !changing {
                logger::warn(&format!(
                    "You can't change this column type, column: `{}`",
                    column.name
                ));
            }
            if !changing {
                // every column of a multi-column type (timestamps, morphs) needs its own ADD COLUMN
                defs = defs
                    .into_iter()
                    .map(|d| format!("ADD COLUMN {}", d))
                    .collect();
            }
            footers = footers.into_iter().map(|f| format!("ADD {}", f)).collect();
        } else if column.change {
            logger::warn(&format!(
                "You can't change column while trying to create table, column: `{}`",
                column.name
            ));
        }

        (
            defs.join(",\n"),
            footers.join(",\n"),
            post.join("\n"),
            String::from(",\n"),
        )
    }

    fn foreign_key(&self, key: &ForeignKey, table_name: &str, action: &TableAction) -> String {
        let update = match key.on_update {
            true => "ON UPDATE CASCADE",
            false => "",
        };
        let delete = match key.on_delete {
            true => "ON DELETE CASCADE",
            false => "",
        };

        let prefix = match *action {
            TableAction::Alter => "ADD ",
            _ => "",
        };
        format!(
            "{} CONSTRAINT {} FOREIGN KEY ({}) REFERENCES {} ({}) {} {}",
            prefix,
            ident(&format!("{}_{}_foreign", table_name, key.column_name)),
            ident(&key.column_name),
            ident(&key.foreign_table),
            ident(&key.referenced_column),
            update,
            delete
        )
    }

    fn drop_column(&self, _table_name: &str, column_name: &str) -> String {
        format!("DROP COLUMN {}", ident(column_name))
    }

    fn table_sql(
        &self,
        table_name: &str,
        body_sql: &str,
        post_sql: &str,
        action: &TableAction,
    ) -> String {
        match action {
            // No table-level COLLATE / ENGINE in PostgreSQL
            TableAction::Create => {
                format!(
                    "CREATE TABLE {} (\n {} \n);\n {}",
                    ident(table_name),
                    body_sql,
                    post_sql
                )
            }
            TableAction::Alter => {
                format!(
                    "ALTER TABLE {} \n {} ;\n {}",
                    ident(table_name),
                    body_sql,
                    post_sql
                )
            }
            _ => "".to_string(),
        }
    }

    // sqlx + PostgreSQL uses numbered placeholders ($1, $2) instead of `?`
    fn get_ran(&self, table: &str) -> String {
        format!(r#"SELECT "migration" FROM {}"#, ident(table))
    }

    fn get_ran_gt(&self, table: &str) -> String {
        format!(r#"SELECT "migration" FROM {} WHERE "batch" > $1"#, ident(table))
    }

    fn get_next_batch_number(&self, table: &str) -> String {
        format!(r#"SELECT MAX("batch")::BIGINT AS "batch" FROM {}"#, ident(table))
    }

    fn add_migrated_table(&self, table: &str) -> String {
        format!(
            r#"INSERT INTO {} ("migration", "batch") VALUES ($1,  CAST($2 AS INTEGER))"#,
            ident(table)
        )
    }

    fn rem_migrated_table(&self, table: &str) -> String {
        format!(r#"DELETE FROM {} WHERE "migration" = $1"#, ident(table))
    }

    fn record_exists(&self, table: &str, column: &str) -> String {
        format!(
            r#"SELECT COUNT(*) AS "count" FROM {} WHERE {} = $1"#,
            ident(table),
            ident(column)
        )
    }

    fn record_exists_except(&self, table: &str, column: &str, except: &str) -> String {
        format!(
            r#"SELECT COUNT(*) AS "count" FROM {} WHERE {} = $1 AND {} <> $2"#,
            ident(table),
            ident(column),
            ident(except)
        )
    }

    fn get_db_tables_columns_struct(&self) -> String {
        "SELECT table_name || '.' || column_name AS name
        FROM information_schema.columns
        WHERE table_schema NOT IN ('information_schema', 'pg_catalog')
        ORDER BY table_schema, table_name, ordinal_position;".into()
    }
}