use std::path::StripPrefixError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("io error")]
    Io(#[from] std::io::Error),
    #[error("failed to execute lua-language-server")]
    Exec,
    #[error("unable to parse doc json")]
    JsonParse(#[from] serde_json::Error),
    #[error("file path is not inside the workspace")]
    PathPrefix(#[from] StripPrefixError),
}