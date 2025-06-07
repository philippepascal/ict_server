use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ICTError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("Invalid UUID")]
    Uuid(#[from] uuid::Error),

    #[error("RSA issue")]
    RSA(#[from] rsa::Error),

    #[error("PKCS issue")]
    PKCS(#[from] rsa::pkcs8::Error),

    #[error("Secret error")]
    Secret(#[from] totp_rs::SecretParseError),

    #[error("Custom error: {0}")]
    Custom(String),
}