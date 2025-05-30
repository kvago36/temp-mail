use thiserror::Error;
use std::io;
// use std::path::PathBuf;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("data store disconnected")]
    Disconnect(#[from] io::Error),
    #[error("unknown data store error")]
    TonicError(#[from] tonic::transport::Error),
    #[error("Unknown SMTP command")]
    UnknownCommand,
    #[error("Waiting for more data to complete")]
    IncompleteData,
    #[error("Error while parsing string")]
    ParseError,
    #[error("State error")]
    StateError,
    #[error("Failed to create directory: {path:?}: {source}")]
    CreateDirError {
        path: String,
        #[source]
        source: io::Error,
    },
    #[error("Failed to create file: {filename:?}: {source}")]
    CreateFileError {
        filename: String,
        #[source]
        source: io::Error,
    },
}
