#[derive(Debug)]
pub struct Table {
    name: String,
    columns: Vec<Column>,
    foreign_keys: Vec<ForeignKey>,
    current: Column,
    current_foreign: ForeignKey,
    comment: String,
    pub action: TableAction,
}

#[derive(Debug)]
pub enum TableAction {
    Create,
    Alter,
    Drop,
    Other,
}

#[derive(Debug, Clone, PartialEq)]
enum ColumnDataType {
    DTId, // add id
    DTBoolean,
    DTString,
    DTJson,
    DTInteger,
    DTTinyInteger,
    DTSmallInteger,
    DTMediumInteger,
    DTBigInteger,
    DTFloat,
    DTDouble,
    DTDecimal,
    DTDate,
    DTTime,
    DTTimestamp,
    DTTimestamps, // to add created_at and updated_at
    DTDateTime,
    DTMorph,
    DTText,
    DTTinyText,
    DTMediumText,
    DTLongText,
    DTSoftDelete,
    DTEnum,
    DTSet,
    DTNone,
}

#[derive(Debug, Clone)]
pub struct Column {
    name: String,
    data_type: ColumnDataType,
    nullable: bool,
    option: ColumnOption,
    comment: String,
    unique: bool,
    index: bool,
    unsigned: bool,
    default: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
enum ColumnOption {
    None,
    Length(i32),
    Precision(i8),
    Values(Vec<String>),
    Float((i8, i8)),
    Index(String),
}

#[derive(Debug, Clone)]
struct ForeignKey {
    column_name: String,
    foreign_table: String,
    referenced_column: String,
    on_delete: bool,
    on_update: bool,
}

macro_rules! warning_invalid_assign {
    ($col:expr) => {
        if $col.data_type == ColumnDataType::DTNone {
            println!(
                "Warning: You are executing the {} function while not having \
            previously specified the column type or name,\
             which is likely to produce an undesirable result.",
                stringify!(func_name)
            );
        }
    };
}

macro_rules! initial_new_column {
    ($self:expr, $column_name:expr,$data_type:expr) => {
        Self::check($self);
        $self.current.name = $column_name.trim().to_string();
        $self.current.data_type = $data_type;
    };
}

#[allow(dead_code)]
impl Table {
    pub fn new(table_name: &str) -> Self {
        Self {
            name: table_name.to_string(),
            columns: Vec::new(),
            foreign_keys: Vec::new(),
            current: Column {
                name: String::new(),
                data_type: ColumnDataType::DTNone,
                nullable: false,
                option: ColumnOption::None,
                comment: String::new(),
                default: String::new(),
                unique: false,
                index: false,
                unsigned: false,
            },
            current_foreign: ForeignKey {
                column_name: String::new(),
                referenced_column: String::new(),
                foreign_table: String::new(),
                on_delete: false,
                on_update: false,
            },
            comment: String::new(),
            action: TableAction::Other,
        }
    }

    fn check(&mut self) {
        if self.current.data_type != ColumnDataType::DTNone {
            self.columns.push(self.current.clone());
            self.current.reset();
        }
    }

    fn check_foreign(&mut self) {
        if self.current_foreign.column_name != ""
            && self.current_foreign.foreign_table != ""
            && self.current_foreign.referenced_column != ""
        {
            self.foreign_keys.push(self.current_foreign.clone());
            self.current_foreign.reset();
        }
    }

    fn finalize(&mut self) {
        if self.current.data_type != ColumnDataType::DTNone {
            self.columns.push(self.current.clone());
            self.current.reset();
        }
    }

    // --------------------------------------------------------------------------------------------

    pub fn id(&mut self) -> &mut Self {
        initial_new_column!(self, "id", ColumnDataType::DTId);
        self.current.unsigned = true;
        self.current.nullable = false;
        self
    }

    pub fn boolean(&mut self, column_name: &str) -> &mut Self {
        initial_new_column!(self, column_name, ColumnDataType::DTBoolean);
        self
    }

    pub fn string(&mut self, column_name: &str, length: i32) -> &mut Self {
        initial_new_column!(self, column_name, ColumnDataType::DTString);
        self.current.option = ColumnOption::Length(length);
        self
    }

    pub fn text(&mut self, column_name: &str) -> &mut Self {
        initial_new_column!(self, column_name, ColumnDataType::DTText);
        self.current.data_type = ColumnDataType::DTText;
        self
    }

    pub fn tiny_text(&mut self, column_name: &str) -> &mut Self {
        initial_new_column!(self, column_name, ColumnDataType::DTTinyText);
        self
    }

    pub fn medium_text(&mut self, column_name: &str) -> &mut Self {
        initial_new_column!(self, column_name, ColumnDataType::DTMediumText);
        self
    }
    pub fn long_text(&mut self, column_name: &str) -> &mut Self {
        initial_new_column!(self, column_name, ColumnDataType::DTLongText);
        self
    }

    pub fn json(&mut self, column_name: &str) -> &mut Self {
        initial_new_column!(self, column_name, ColumnDataType::DTJson);
        self
    }

    pub fn integer(&mut self, column_name: &str) -> &mut Self {
        initial_new_column!(self, column_name, ColumnDataType::DTInteger);
        self
    }

    pub fn tiny_integer(&mut self, column_name: &str) -> &mut Self {
        initial_new_column!(self, column_name, ColumnDataType::DTTinyInteger);
        self
    }

    pub fn small_integer(&mut self, column_name: &str) -> &mut Self {
        initial_new_column!(self, column_name, ColumnDataType::DTSmallInteger);
        self
    }

    pub fn medium_integer(&mut self, column_name: &str) -> &mut Self {
        initial_new_column!(self, column_name, ColumnDataType::DTMediumInteger);
        self
    }

    pub fn big_integer(&mut self, column_name: &str) -> &mut Self {
        initial_new_column!(self, column_name, ColumnDataType::DTBigInteger);
        self
    }

    pub fn double(&mut self, column_name: &str) -> &mut Self {
        initial_new_column!(self, column_name, ColumnDataType::DTDouble);
        self
    }

    pub fn float(&mut self, column_name: &str, precision: i8) -> &mut Self {
        initial_new_column!(self, column_name, ColumnDataType::DTFloat);
        self.current.option = ColumnOption::Precision(precision);
        self
    }

    pub fn decimal(&mut self, column_name: &str, total: i8, place: i8) -> &mut Self {
        initial_new_column!(self, column_name, ColumnDataType::DTDecimal);
        self.current.option = ColumnOption::Float((total, place));
        self
    }

    pub fn date(&mut self, column_name: &str) -> &mut Self {
        initial_new_column!(self, column_name, ColumnDataType::DTDate);
        self
    }

    pub fn datetime(&mut self, column_name: &str) -> &mut Self {
        initial_new_column!(self, column_name, ColumnDataType::DTDateTime);
        self
    }

    pub fn time(&mut self, column_name: &str) -> &mut Self {
        initial_new_column!(self, column_name, ColumnDataType::DTTime);
        self
    }

    pub fn timestamp(&mut self, column_name: &str) -> &mut Self {
        initial_new_column!(self, column_name, ColumnDataType::DTTimestamp);
        self
    }

    pub fn timestamps(&mut self) -> &mut Self {
        Self::check(self);
        self.current.data_type = ColumnDataType::DTTimestamps;
        self
    }

    pub fn soft_delete(&mut self) -> &mut Self {
        initial_new_column!(self, "deleted_at", ColumnDataType::DTSoftDelete);
        self.current.nullable = true;
        self
    }

    pub fn morph(&mut self, column_name: &str, index_name: &str) -> &mut Self {
        initial_new_column!(self, column_name, ColumnDataType::DTMorph);
        self.current.option = ColumnOption::Index(index_name.trim().to_string());
        self.current.nullable = false;
        self
    }

    pub fn nullable_morphs(&mut self, column_name: &str, index_name: &str) -> &mut Self {
        initial_new_column!(self, column_name, ColumnDataType::DTMorph);
        self.current.option = ColumnOption::Index(index_name.trim().to_string());
        self.current.nullable = true;
        self
    }

    pub fn enums(&mut self, column_name: &str, values: Vec<String>) -> &mut Self {
        initial_new_column!(self, column_name, ColumnDataType::DTEnum);
        self.current.option = ColumnOption::Values(values);
        self
    }

    pub fn sets(&mut self, column_name: &str, values: Vec<String>) -> &mut Self {
        initial_new_column!(self, column_name, ColumnDataType::DTSet);
        self.current.option = ColumnOption::Values(values);
        self
    }

    // --------------------------------------------------------------------------------------------

    pub fn foreign(&mut self, column_name: &str) -> &mut Self {
        self.check_foreign();
        self.current_foreign.column_name = column_name.to_string();
        self
    }

    pub fn reference(&mut self, referenced_column: &str) -> &mut Self {
        self.current_foreign.referenced_column = referenced_column.to_string();
        self
    }

    pub fn on(&mut self, referenced_table_name: &str) -> &mut Self {
        self.current_foreign.foreign_table = referenced_table_name.to_string();
        self
    }

    pub fn cascade_on_delete(&mut self) -> &mut Self {
        self.current_foreign.on_delete = true;
        self
    }

    pub fn cascade_on_update(&mut self) -> &mut Self {
        self.current_foreign.on_update = true;
        self
    }

    // --------------------------------------------------------------------------------------------

    pub fn nullable(&mut self) -> &mut Self {
        warning_invalid_assign!(&self.current);
        self.current.nullable = true;
        self
    }

    pub fn index(&mut self) -> &mut Self {
        self.current.index = true;
        self
    }

    pub fn unique(&mut self) -> &mut Self {
        self.current.unique = true;
        self
    }

    pub fn unsigned(&mut self) -> &mut Self {
        self.current.unsigned = true;
        self
    }

    pub fn comment(&mut self, comment: &str) -> &mut Self {
        if self.current.data_type == ColumnDataType::DTNone {
            self.comment = comment.trim().to_string();
        } else {
            self.current.comment = comment.trim().to_string();
        }
        self
    }

    pub fn default(&mut self, default_value: &str) -> &mut Self {
        self.current.default = default_value.trim().to_string();
        self
    }

    // --------------------------------------------------------------------------------------------

    pub fn validate(&mut self) -> &mut Self {
        self.check();
        self.check_foreign();


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
    // --------------------------------------------------------------------------------------------
}

impl Column {
    fn reset(&mut self) {
        self.name = String::new();
        self.data_type = ColumnDataType::DTNone;
        self.nullable = false;
        self.option = ColumnOption::None;
        self.comment = String::new();
        self.default = String::new();
        self.unique = false;
        self.index = false;
        self.unsigned = false;
    }

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

            ColumnDataType::DTBoolean =>{
                if self.unique {
                    return false;
                }
                if self.default != "false" && self.default != "true" && self.default != "1" && self.default != "0" {
                    return false;
                }
            }
            _ => return true,
        }

        true
    }
}

impl ForeignKey {
    fn reset(&mut self) {
        self.column_name = String::new();
        self.referenced_column = String::new();
        self.foreign_table = String::new();
        self.on_delete = false;
        self.on_update = false;
    }

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
