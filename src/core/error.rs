use std::fmt;

use thiserror::Error;

pub type HniResult<T> = Result<T, HniError>;

#[derive(Debug, Error)]
pub enum HniError {
    #[error("parse error: {0}")]
    Parse(String),
    #[error("config error: {0}")]
    Config(String),
    #[error("detection error: {0}")]
    Detection(String),
    #[error("execution error: {0}")]
    Execution(String),
    #[error("interactive error: {0}")]
    Interactive(String),
    #[error("network error: {0}")]
    Network(String),
    #[error("storage error: {0}")]
    Storage(String),
    #[error("{0}")]
    Internal(String),
}

impl HniError {
    pub fn parse(message: impl Into<String>) -> Self {
        Self::Parse(message.into())
    }

    pub fn config(message: impl Into<String>) -> Self {
        Self::Config(message.into())
    }

    pub fn detection(message: impl Into<String>) -> Self {
        Self::Detection(message.into())
    }

    pub fn execution(message: impl Into<String>) -> Self {
        Self::Execution(message.into())
    }

    pub fn interactive(message: impl Into<String>) -> Self {
        Self::Interactive(message.into())
    }

    pub fn network(message: impl Into<String>) -> Self {
        Self::Network(message.into())
    }

    pub fn storage(message: impl Into<String>) -> Self {
        Self::Storage(message.into())
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }

    pub fn with_context(self, context: impl fmt::Display) -> Self {
        match self {
            Self::Parse(message) => Self::Parse(format!("{context}: {message}")),
            Self::Config(message) => Self::Config(format!("{context}: {message}")),
            Self::Detection(message) => Self::Detection(format!("{context}: {message}")),
            Self::Execution(message) => Self::Execution(format!("{context}: {message}")),
            Self::Interactive(message) => Self::Interactive(format!("{context}: {message}")),
            Self::Network(message) => Self::Network(format!("{context}: {message}")),
            Self::Storage(message) => Self::Storage(format!("{context}: {message}")),
            Self::Internal(message) => Self::Internal(format!("{context}: {message}")),
        }
    }
}

impl From<anyhow::Error> for HniError {
    fn from(value: anyhow::Error) -> Self {
        Self::internal(value.to_string())
    }
}
