use std::io;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{context}: {source}")]
    Io { context: String, source: io::Error },

    #[error("{0}")]
    Validation(String),

    #[error("compile failed:\n{0}")]
    CompileFailed(String),

    #[error("{0}")]
    Image(String),
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn io_err(context: impl Into<String>, source: io::Error) -> Error {
    Error::Io {
        context: context.into(),
        source,
    }
}
