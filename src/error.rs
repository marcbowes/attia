use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProgramError {
    #[error("URL not as expected, got `{0}` but expected `{1}`")]
    WrongUrl(String, String),
    #[error("{0}")]
    Unexpected(String),
    #[error("IO error: {source}")]
    IoError {
        #[from]
        source: std::io::Error,
    },
    #[error(transparent)]
    TomlSerError {
        #[from]
        source: toml::ser::Error,
    },
    #[error(transparent)]
    TomlDeError {
        #[from]
        source: toml::de::Error,
    },
    #[error(transparent)]
    SerdeJsonError {
        #[from]
        source: serde_json::Error,
    },
    #[error(transparent)]
    FantocciniCmdError {
        #[from]
        source: fantoccini::error::CmdError,
        //backtrace: std::backtrace::Backtrace,
    },
    #[error(transparent)]
    ScraperUsageError {
        #[from]
        source: scraper::error::SelectorErrorKind<'static>,
    },
    #[error(transparent)]
    TantivyError {
        #[from]
        source: tantivy::TantivyError,
    },
    #[error(transparent)]
    TantivyUsageError {
        #[from]
        source: tantivy::query::QueryParserError,
    },
    #[error(transparent)]
    TantivyDirectoryError {
        #[from]
        source: tantivy::directory::error::OpenDirectoryError,
    },
}

pub type Result<T> = std::result::Result<T, ProgramError>;
