use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParasiteError {}

pub type ParasiteResult<T> = Result<T, ParasiteError>;
