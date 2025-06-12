use std::{io, num::ParseIntError, time::SystemTimeError};
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

    #[error("TOTP error")]
    TOTP(#[from] totp_rs::TotpUrlError),

    #[error("System Time error")]
    SystemTimeError(#[from] SystemTimeError),

    #[error("Base64 decode error")]
    DecodeError(#[from] base64::DecodeError),

    #[error("Parse Int error")]
    ParseIntError(#[from] ParseIntError),

    #[error("PK error")]
    PKError(#[from] rsa::pkcs8::spki::Error),

    #[error("Signature error")]
    SignatureError(#[from] rsa::signature::Error),

    #[error("String error")]
    StringError(#[from] std::string::FromUtf8Error),

    #[error("Custom error: {0}")]
    Custom(String),
}
