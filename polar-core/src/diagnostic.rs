use super::error::PolarError;

pub enum Diagnostic {
    Error(PolarError),
    Warning(String),
}
