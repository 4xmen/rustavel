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

    pub fn add(&mut self, field: &str, message: String) {
        self.errors
            .entry(field.to_string())
            .or_default()
            .push(message);
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }
}

pub trait LaravelValidator : Sync + Send {
    fn validate(&self) -> Result<(), ValidationErrors>;
}


