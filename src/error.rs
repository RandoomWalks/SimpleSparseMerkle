use thiserror::Error;

#[derive(Error, Debug)]
pub enum SMTError {
    #[error("Key-value store error: {0}")]
    KVStoreError(#[from] std::io::Error),

    #[error("Invalid proof")]
    InvalidProof,

    #[error("Unsupported operation")]
    UnsupportedOperation,
}