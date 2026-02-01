use super::generator::SqlGenerator;
use crate::logger;
use crate::db::table::{Column, ColumnDataType, ColumnOption, DefaultValue, ForeignKey, TableAction};

#[derive(Debug)]
pub struct SqliteGenerator;

#[allow(dead_code)]
impl SqliteGenerator {
    #[inline]
    fn master(&self) -> &'static str {
        "sqlite_master"
    }
}

impl SqlGenerator for SqliteGenerator {
    fn drop_table_if_exists(&self, table_name: &str) -> String {
        format!("DROP TABLE IF EXISTS \"{}\";", table_name)
    }

    fn get_tables(&self) -> String {
        "
        SELECT name
        FROM sqlite_master
        WHERE type = 'table'
          AND name NOT LIKE 'sqlite_%'
        "
        .to_string()
    }

    fn get_views(&self) -> String {
        "
        SELECT name
        FROM sqlite_master
        WHERE type = 'view'
        "
        .to_string()
    }

    fn get_column_listing(&self, table_name: &str) -> String {
        format!(
            "
        SELECT p.name AS column_name
        FROM sqlite_master m
        JOIN pragma_table_info(m.name) p
        WHERE m.type = 'table'
          AND m.name NOT LIKE 'sqlite_%'
          AND m.name = '{}'
        ",
            table_name
        )
    }

    fn get_foreign_keys(&self, table_name: &str) -> String {
        format!("PRAGMA foreign_key_list(\"{}\");", table_name)
    }

    fn drop_table(&self, table_name: &str) -> String {
        format!("DROP TABLE \"{}\";", table_name)
    }

    fn drop_all_tables(&self) -> String {
        "
        SELECT 'DROP TABLE IF EXISTS \"' || name || '\";'
        FROM sqlite_master
        WHERE type = 'table'
          AND name NOT LIKE 'sqlite_%';
        "
        .to_string()
    }

    fn drop_all_views(&self) -> String {
        "
        SELECT 'DROP VIEW IF EXISTS \"' || name || '\";'
        FROM sqlite_master
        WHERE type = 'view';
        "
        .to_string()
    }

    fn has_column(&self, table_name: &str, column_name: &str) -> String {
        format!(
            "
            SELECT \"1\"
            FROM pragma_table_info(\"{}\")
            WHERE name = '{}'
            LIMIT 1
            ",
            table_name, column_name
        )
    }

    fn has_table(&self, table_name: &str) -> String {
        format!(
            "
            SELECT \"1\"
            FROM sqlite_master
            WHERE type = 'table'
              AND name = '{}'
            LIMIT 1
            ",
            table_name
        )
    }

    fn has_view(&self, table_name: &str) -> String {
        format!(
            "
            SELECT \"1\"
            FROM sqlite_master
            WHERE type = 'view'
              AND name = '{}'
            LIMIT 1
            ",
            table_name
        )
    }

    fn has_index(&self, table_name: &str, columns_name: Vec<&str>) -> String {
        let cols = columns_name
            .iter()
            .map(|c| format!("'{}'", c))
            .collect::<Vec<_>>()
            .join(",");

        let count = columns_name.len();

        format!(
            "
            SELECT \"1\"
            FROM pragma_index_list('{}') il
            JOIN pragma_index_info(il.name) ii
            WHERE ii.name IN ({})
            GROUP BY il.name
            HAVING COUNT(*) = {}
            LIMIT 1
            ",
            table_name, cols, count
        )
    }

    fn create_database(&self, _db_name: &str) -> String {
        "-- sqlite: database is created on connection".to_string()
    }

    fn drop_database_if_exists(&self, _db_name: &str) -> String {
        "-- sqlite: drop database not supported via SQL".to_string()
    }

    fn disable_foreign_key_constraints(&self, _db_name: &str) -> String {
        "PRAGMA foreign_keys = OFF;".to_string()
    }

    fn enable_foreign_key_constraints(&self, _db_name: &str) -> String {
        "PRAGMA foreign_keys = ON;".to_string()
    }

    fn rename(&self, old_table_name: &str, new_table_name: &str) -> String {
        format!(
            "ALTER TABLE \"{}\" RENAME TO \"{}\";",
            old_table_name, new_table_name
        )
    }

    fn column(
        &self,
        column: &Column,
        table_name: &str,
        action: &TableAction,
    ) -> (String, String, String) {
        let mut column_sql = String::new();
        let mut footer_sql = String::new();
        let mut post_sql = String::new();

        let nullable = if column.nullable { "" } else { "NOT NULL" };

        let def = match &column.default {
            DefaultValue::Null => "",
            DefaultValue::String(val) => &format!("DEFAULT '{}'", val),
            DefaultValue::JsonArray => "DEFAULT '[]'",
            DefaultValue::CurrenTimestamp => "DEFAULT CURRENT_TIMESTAMP",
            DefaultValue::Bool(v) => {
                if *v {
                    "DEFAULT 1"
                } else {
                    "DEFAULT 0"
                }
            }
            DefaultValue::Int(v) => &format!("DEFAULT {}", v),
            _ => "",
        };

        match column.data_type {
            ColumnDataType::DTId => {
                column_sql = "`id` integer NOT NULL".to_string();
                footer_sql = "PRIMARY KEY(`id` AUTOINCREMENT)".to_string();
            }

            ColumnDataType::DTBoolean => {
                column_sql = format!("`{}` boolean {} {}", column.name, nullable, def);
            }

            ColumnDataType::DTTinyInteger
            | ColumnDataType::DTInteger
            | ColumnDataType::DTSmallInteger
            | ColumnDataType::DTMediumInteger
            | ColumnDataType::DTBigInteger => {
                column_sql = format!("`{}` integer {} {}", column.name, nullable, def);
            }

            ColumnDataType::DTFloat | ColumnDataType::DTDouble => {
                column_sql = format!("`{}` float {} {}", column.name, nullable, def);
            }

            ColumnDataType::DTDecimal => {
                let (p, s) = match &column.option {
                    ColumnOption::Float((p, s)) => (p, s),
                    _ => (&20, &6),
                };
                column_sql = format!(
                    "`{}` decimal({},{}) {} {}",
                    column.name, p, s, nullable, def
                );
            }

            ColumnDataType::DTString => {
                let len = match column.option {
                    ColumnOption::Length(l) => l,
                    _ => 255,
                };
                column_sql = format!("`{}` varchar({}) {} {}", column.name, len, nullable, def);
            }

            ColumnDataType::DTText
            | ColumnDataType::DTTinyText
            | ColumnDataType::DTMediumText
            | ColumnDataType::DTLongText => {
                column_sql = format!("`{}` text {} {}", column.name, nullable, def);
            }

            ColumnDataType::DTJson => {
                column_sql = format!(
                    "`{}` json {} {} CHECK(json_valid(`{}`))",
                    column.name, nullable, def, column.name
                );
            }

            ColumnDataType::DTDate
            | ColumnDataType::DTDateTime
            | ColumnDataType::DTTime
            | ColumnDataType::DTTimestamp => {
                column_sql = format!("`{}` datetime {} {}", column.name, nullable, def);
            }

            ColumnDataType::DTTimestamps => {
                column_sql = "`created_at` datetime, `updated_at` datetime".to_string();
            }

            ColumnDataType::DTSoftDelete => {
                column_sql = "`deleted_at` datetime".to_string();
            }

            ColumnDataType::DTEnum | ColumnDataType::DTSet => {
                let values = match &column.option {
                    ColumnOption::Values(items) => items
                        .iter()
                        .map(|v| format!("'{}'", v))
                        .collect::<Vec<_>>()
                        .join(", "),
                    _ => String::new(),
                };

                column_sql = format!(
                    "`{}` varchar {} {} CHECK(`{}` IN ({}))",
                    column.name, nullable, def, column.name, values
                );
            }

            ColumnDataType::DTMorph => {
                column_sql = format!(
                    "`{}_type` varchar(255) {} {}, `{}`_id integer {} {}",
                    column.name, nullable, def, column.name, nullable, def
                );

                post_sql = format!(
                    "CREATE INDEX `morph_{}_type_{}_id_index`
                 ON `{}` (`{}_type`, `{}_id`)",
                    column.name, column.name, table_name, column.name, column.name
                );
            }

            _ => {}
        }

        if column.unique {
            post_sql = format!(
                "CREATE UNIQUE INDEX `{}_{}_unique`
             ON `{}` (`{}`)",
                table_name, column.name, table_name, column.name
            );
        } else if column.index {
            post_sql = format!(
                "CREATE INDEX `{}_{}_index`
             ON `{}` (`{}`)",
                table_name, column.name, table_name, column.name
            );
        }

        if *action == TableAction::Alter {
            if column.change {
                logger::warn("SQLite does not support CHANGE COLUMN");
            } else {
                column_sql = format!("ADD COLUMN {}", column_sql);
            }
        }

        (column_sql, footer_sql, post_sql)
    }

    fn foreign_key(&self, key: &ForeignKey, _table_name: &str, _action: &TableAction) -> String {
        let update = if key.on_update {
            "on update cascade"
        } else {
            ""
        };

        let delete = if key.on_delete {
            "on delete cascade"
        } else {
            ""
        };

        format!(
            "FOREIGN KEY(\"{}\") REFERENCES \"{}\"(\"{}\") {} {} ",
            key.column_name, key.foreign_table, key.column_name, update, delete
        )
    }
    fn drop_column(&self, column_name: &str) -> String {
        format!(
            "-- SQLite does not support DROP COLUMN directly: `{}`",
            column_name
        )
    }

    fn table_sql(
        &self,
        table_name: &str,
        body_sql: &str,
        post_sql: &str,
        action: &TableAction,
    ) -> String {
        match action {
            TableAction::Create => {
                format!(
                    "CREATE TABLE `{}` ( {} ) \n ;\n {}",
                    table_name, body_sql, post_sql
                )
            }
            TableAction::Alter => {
                format!(
                    "ALTER TABLE `{} \n {} ; \n {}`",
                    table_name, body_sql, post_sql
                )
            }
            _ => "".to_string(),
        }
    }
}
