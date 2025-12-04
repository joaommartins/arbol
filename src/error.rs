use thiserror::Error;

pub type Result<T, E = ArbolError> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum ArbolError {
    #[error("IO: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("Tree-sitter: failed to set language: {0}")]
    SetLanguage(String),
    #[error("Tree-sitter: parse failed")]
    ParseFailed,
    #[error("Query compile error")]
    QueryCompile,
    #[error("CLI: {0}")]
    Cli(String),
}
