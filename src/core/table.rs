#[derive(Debug)]
pub struct Table {
    pub name: String,
    pub columns: Vec<Column>,
    pub foreign_keys: Vec<ForeignKey>,
    pub comment: String,
    pub action: TableAction,
    pub drop_columns: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub enum TableAction {
    None,
    Create,
    Alter,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ColumnDataType {
    DTId, // add id
    DTBoolean,
    DTTinyInteger,
    DTInteger,
    DTSmallInteger,
    DTMediumInteger,
    DTBigInteger,
    DTFloat,
    DTDouble,
    DTDecimal,
    DTString,
    DTText,
    DTTinyText,
    DTMediumText,
    DTLongText,
    DTJson,
    DTDate,
    DTDateTime,
    DTTime,
    DTTimestamp,
    DTTimestamps, // to add created_at and updated_at
    DTSoftDelete,
    DTEnum,
    DTSet,
    DTMorph,
    DTNone,
}

#[derive(Debug, Clone)]
pub struct Column {
    pub name: String,
    pub data_type: ColumnDataType,
    pub nullable: bool,
    pub option: ColumnOption,
    pub comment: String,
    pub unique: bool,
    pub index: bool,
    pub unsigned: bool,
    pub default: DefaultValue,
    pub change: bool,
    pub collation: String,
}

pub struct ColumnBuilder<'a> {
    table: &'a mut Table,
    column: Column,
}
pub struct ForeignKeyBuilder<'a> {
    table: &'a mut Table,
    key: ForeignKey,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum ColumnOption {
    None,
    Length(i32),
    Precision(i8),
    Values(Vec<String>),
    Float((i8, i8)),
    Index(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum DefaultValue {
    None,
    Null,
    JsonArray,
    Bool(bool),
    Int(i64),
    String(String),
    CurrenTimestamp,
}

#[derive(Debug, Clone)]
pub struct ForeignKey {
    pub column_name: String,
    pub foreign_table: String,
    pub referenced_column: String,
    pub on_delete: bool,
    pub on_update: bool,
}

#[allow(dead_code)]
impl Table {
    pub fn new(table_name: &str) -> Self {
        Self {
            name: table_name.to_string(),
            columns: Vec::new(),
            foreign_keys: Vec::new(),
            comment: String::new(),
            action: TableAction::None,
            drop_columns: Vec::new(),
        }
    }

    pub fn table_comment(&mut self, comment: impl Into<String>) -> &mut Self {
        self.comment = comment.into();
        self
    }

    fn column(
        &mut self,
        name: impl Into<String>,
        column_data: ColumnDataType,
        option: ColumnOption,
    ) -> ColumnBuilder<'_> {
        ColumnBuilder {
            table: self,
            column: Column {
                name: name.into(),
                data_type: column_data,
                option,
                nullable: false,
                unique: false,
                index: false,
                default: DefaultValue::None,
                comment: String::new(),
                unsigned: false,
                change: false,
                collation: String::new(),
            },
        }
    }

    // --------------------------------------------------------------------------------------------

    pub fn id(&mut self) -> ColumnBuilder<'_> {
        self.column("id", ColumnDataType::DTId, ColumnOption::None)
    }

    pub fn boolean(&mut self, name: impl Into<String>) -> ColumnBuilder<'_> {
        self.column(name, ColumnDataType::DTBoolean, ColumnOption::None)
    }

    pub fn string(&mut self, name: impl Into<String>, len: i32) -> ColumnBuilder<'_> {
        self.column(name, ColumnDataType::DTString, ColumnOption::Length(len))
    }

    pub fn text(&mut self, name: impl Into<String>) -> ColumnBuilder<'_> {
        self.column(name, ColumnDataType::DTText, ColumnOption::None)
    }

    pub fn tiny_text(&mut self, name: impl Into<String>) -> ColumnBuilder<'_> {
        self.column(name, ColumnDataType::DTTinyText, ColumnOption::None)
    }

    pub fn medium_text(&mut self, name: impl Into<String>) -> ColumnBuilder<'_> {
        self.column(name, ColumnDataType::DTMediumText, ColumnOption::None)
    }
    pub fn long_text(&mut self, name: impl Into<String>) -> ColumnBuilder<'_> {
        self.column(name, ColumnDataType::DTLongText, ColumnOption::None)
    }

    pub fn json(&mut self, name: impl Into<String>) -> ColumnBuilder<'_> {
        self.column(name, ColumnDataType::DTJson, ColumnOption::None)
    }

    pub fn integer(&mut self, name: impl Into<String>) -> ColumnBuilder<'_> {
        self.column(name, ColumnDataType::DTInteger, ColumnOption::None)
    }

    pub fn tiny_integer(&mut self, name: impl Into<String>) -> ColumnBuilder<'_> {
        self.column(name, ColumnDataType::DTTinyInteger, ColumnOption::None)
    }

    pub fn small_integer(&mut self, name: impl Into<String>) -> ColumnBuilder<'_> {
        self.column(name, ColumnDataType::DTSmallInteger, ColumnOption::None)
    }

    pub fn medium_integer(&mut self, name: impl Into<String>) -> ColumnBuilder<'_> {
        self.column(name, ColumnDataType::DTMediumInteger, ColumnOption::None)
    }

    pub fn big_integer(&mut self, name: impl Into<String>) -> ColumnBuilder<'_> {
        self.column(name, ColumnDataType::DTBigInteger, ColumnOption::None)
    }

    pub fn double(&mut self, name: impl Into<String>) -> ColumnBuilder<'_> {
        self.column(name, ColumnDataType::DTDouble, ColumnOption::None)
    }

    pub fn float(&mut self, name: impl Into<String>, precision: i8) -> ColumnBuilder<'_> {
        self.column(
            name,
            ColumnDataType::DTFloat,
            ColumnOption::Precision(precision),
        )
    }

    pub fn decimal(&mut self, name: impl Into<String>, total: i8, place: i8) -> ColumnBuilder<'_> {
        self.column(
            name,
            ColumnDataType::DTDecimal,
            ColumnOption::Float((total, place)),
        )
    }

    pub fn date(&mut self, name: impl Into<String>) -> ColumnBuilder<'_> {
        self.column(name, ColumnDataType::DTDate, ColumnOption::None)
    }

    pub fn datetime(&mut self, name: impl Into<String>) -> ColumnBuilder<'_> {
        self.column(name, ColumnDataType::DTDateTime, ColumnOption::None)
    }

    pub fn time(&mut self, name: impl Into<String>) -> ColumnBuilder<'_> {
        self.column(name, ColumnDataType::DTTime, ColumnOption::None)
    }

    pub fn timestamp(&mut self, name: impl Into<String>) -> ColumnBuilder<'_> {
        self.column(name, ColumnDataType::DTTimestamp, ColumnOption::None)
    }

    pub fn timestamps(&mut self) -> ColumnBuilder<'_> {
        self.column("", ColumnDataType::DTTimestamps, ColumnOption::None)
    }

    pub fn soft_delete(&mut self) -> ColumnBuilder<'_> {
        self.column(
            "deleted_at",
            ColumnDataType::DTSoftDelete,
            ColumnOption::None,
        )
    }

    pub fn morph(
        &mut self,
        name: impl Into<String>,
        index_name: impl Into<String>,
    ) -> ColumnBuilder<'_> {
        self.column(
            name,
            ColumnDataType::DTMorph,
            ColumnOption::Index(index_name.into()),
        )
    }

    pub fn nullable_morphs(
        &mut self,
        name: impl Into<String>,
        index_name: impl Into<String>,
    ) -> ColumnBuilder<'_> {
        self.column(
            name,
            ColumnDataType::DTMorph,
            ColumnOption::Index(index_name.into()),
        )
    }

    pub fn enums<I, S>(&mut self, name: impl Into<String>, values: I) -> ColumnBuilder<'_>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let values = values.into_iter().map(Into::into).collect();
        self.column(name, ColumnDataType::DTEnum, ColumnOption::Values(values))
    }

    pub fn sets<I, S>(&mut self, name: impl Into<String>, values: I) -> ColumnBuilder<'_>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let values = values.into_iter().map(Into::into).collect();
        self.column(name, ColumnDataType::DTSet, ColumnOption::Values(values))
    }

    // --------------------------------------------------------------------------------------------

    pub fn foreign(&mut self, name: impl Into<String>) -> ForeignKeyBuilder<'_> {
        ForeignKeyBuilder {
            table: self,
            key: ForeignKey {
                column_name: name.into(),
                foreign_table: String::new(),
                referenced_column: String::new(),
                on_update: false,
                on_delete: false,
            },
        }
    }

    // --------------------------------------------------------------------------------------------

    pub fn validate(&mut self) -> &mut Self {
        for foreign_key in &mut self.foreign_keys {
            if !foreign_key.validate() {
                println!("invalid foreign key:");
                dbg!(&foreign_key);
            }
        }
        for column in &mut self.columns {
            if !column.validate() {
                println!("invalid column:");
                dbg!(&column);
            }
        }
        self
    }

    pub fn drop_column(&mut self, name: impl Into<String>) {
        self.drop_columns.push(name.into());
    }
    // --------------------------------------------------------------------------------------------
}

impl Column {
    fn validate(&mut self) -> bool {
        match self.data_type {
            ColumnDataType::DTNone => return false,

            ColumnDataType::DTString
            | ColumnDataType::DTLongText
            | ColumnDataType::DTMediumText
            | ColumnDataType::DTTinyText
            | ColumnDataType::DTJson => {
                // String types cannot be unsigned
                if self.unsigned {
                    return false;
                }

                // Handle Length option
                if let ColumnOption::Length(length) = &self.option {
                    if *length <= 0 {
                        return false;
                    }

                    // If length > 255, cannot be indexed or unique
                    if *length > 255 && (self.index || self.unique) {
                        return false;
                    }
                } else {
                    // If no Length and it's indexed or unique, invalid
                    if self.index || self.unique {
                        return false;
                    }
                }
            }

            ColumnDataType::DTBoolean => {
                if self.unique {
                    return false;
                }
            }
            _ => return true,
        }

        true
    }

    pub fn is_string_type(&self) -> bool {
        match self.data_type {
            ColumnDataType::DTString |
            ColumnDataType::DTLongText |
            ColumnDataType::DTMediumText |
            ColumnDataType::DTTinyText |
            ColumnDataType::DTJson => true,
            _ => false,
        }
    }
}

impl Default for Column {
    fn default() -> Self {
        Column {
            name: String::new(),
            data_type: ColumnDataType::DTNone,
            nullable: false,
            comment: String::new(),
            unique: false,
            index: false,
            default: DefaultValue::None,
            unsigned: false,
            option: ColumnOption::None,
            change: false,
            collation: String::new(),
        }
    }
}
impl Default for ForeignKey {
    fn default() -> Self {
        ForeignKey {
            referenced_column: String::new(),
            column_name: String::new(),
            foreign_table: String::new(),
            on_delete: false,
            on_update: false,
        }
    }
}

impl ForeignKey {
    fn validate(&mut self) -> bool {
        let mut message = String::new();
        if self.column_name.is_empty()
            || self.referenced_column.is_empty()
            || self.foreign_table.is_empty()
        {
            return false;
        }

        true
    }
}

impl<'a> ColumnBuilder<'a> {
    pub fn nullable(mut self) -> Self {
        self.column.nullable = true;
        self
    }

    pub fn unique(mut self) -> Self {
        self.column.unique = true;
        self
    }

    pub fn index(mut self) -> Self {
        self.column.index = true;
        self
    }

    pub fn unsigned(mut self) -> Self {
        self.column.unsigned = true;
        self
    }

    pub fn comment(mut self, comment: impl Into<String>) -> Self {
        self.column.comment = comment.into();
        self
    }

    pub fn default_bool(mut self, value: bool) -> Self {
        self.column.default = DefaultValue::Bool(value);
        self
    }

    pub fn default_int(mut self, value: i64) -> Self {
        self.column.default = DefaultValue::Int(value);
        self
    }

    pub fn default_str(mut self, value: impl Into<String>) -> Self {
        self.column.default = DefaultValue::String(value.into());
        self
    }
    pub fn default_json_array(mut self) -> Self {
        self.column.default = DefaultValue::JsonArray;
        self
    }
    pub fn default_null(mut self) -> Self {
        self.column.default = DefaultValue::Null;
        self
    }

    pub fn default_current_timestamp(mut self) -> Self {
        self.column.default = DefaultValue::CurrenTimestamp;
        self
    }

    pub fn change(mut self) {
        self.column.change = true;
    }

    pub fn collation(mut self, collation: impl Into<String>) -> Self {
        // just work in mysql
        self.column.collation = collation.into();
        self
    }
}

impl<'a> ForeignKeyBuilder<'a> {
    pub fn reference(&mut self, referenced_column: impl Into<String>) -> &mut Self {
        self.key.referenced_column = referenced_column.into();
        self
    }

    pub fn on(&mut self, referenced_table_name: impl Into<String>) -> &mut Self {
        self.key.foreign_table = referenced_table_name.into();
        self
    }

    pub fn cascade_on_delete(&mut self) -> &mut Self {
        self.key.on_delete = true;
        self
    }

    pub fn cascade_on_update(&mut self) -> &mut Self {
        self.key.on_update = true;
        self
    }
}

impl<'a> Drop for ColumnBuilder<'a> {
    fn drop(&mut self) {
        self.table.columns.push(std::mem::take(&mut self.column));
    }
}
impl<'a> Drop for ForeignKeyBuilder<'a> {
    fn drop(&mut self) {
        self.table.foreign_keys.push(std::mem::take(&mut self.key));
    }
}
