use std::collections::HashMap;
use serde::Serialize;

#[derive(Debug,Serialize)]
pub struct ValidationErrors {
    pub errors: HashMap<String, Vec<String>>,
}

impl ValidationErrors {
    pub fn new() -> Self {
        Self {
            errors: HashMap::new(),
        }
    }

    pub fn add(&mut self, field: impl Into<String>, message: impl Into<String>) {
        self.errors
            .entry(field.into())
            .or_default()
            .push(message.into());
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }
}

pub trait LaravelValidator : Sync + Send {
    fn validate(&self) -> Result<(), ValidationErrors>;
}

pub fn is_valid_email(value: &str) -> bool {
    value.contains("@") // so simple
}



