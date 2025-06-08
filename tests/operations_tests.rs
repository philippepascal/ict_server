use ict_server::{ict_db::{Db, Device}, ict_errors::ICTError};
use rand::rngs::OsRng;
use rsa::{pkcs1v15::Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};
use std::thread;
use std::time::Duration;
use totp_rs::{Secret, TOTP};
use uuid::Uuid;

#[test]
fn test_happy_path() -> Result<(), ICTError> {
    let db = Db::new_test_db()?;

    //1 create a device
    //2 register device and collect secret
    //3 authorize device
    //4 generate totp using device and secret
    //5 validate totp

    Ok(())
}