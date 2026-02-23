use std::io;
use minijinja::{ Error as TemplateError};
#[derive(Debug)]
#[allow(dead_code)]
pub enum MakeError {
    Template(TemplateError),
    Io(io::Error),
}


impl From<TemplateError> for MakeError {
    fn from(err: TemplateError) -> Self {
        Self::Template(err)
    }
}

impl From<io::Error> for MakeError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

