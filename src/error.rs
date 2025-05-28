use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("data store disconnected")]
    Disconnect(#[from] std::io::Error),
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
}
