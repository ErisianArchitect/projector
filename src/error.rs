
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("bincode Decode Error: {0}")]
    BincodeDecodeError(#[from] bincode::error::DecodeError),
    #[error("bincode Encode Error: {0}")]
    BincodeEncodeError(#[from] bincode::error::EncodeError),
    #[error("tempfile Persist Error: {0}")]
    TempfilePersistError(#[from] tempfile::PersistError),
    #[error("Temporary Error: {0}")]
    TempErr(&'static str),
}

pub type Result<T> = std::result::Result<T, Error>;