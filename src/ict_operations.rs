use uuid::Uuid;
use rsa::{pkcs8::DecodePrivateKey, RsaPrivateKey};
use totp_rs::{Secret};

use crate::ict_errors::ICTError;
use crate::ict_db::Db;
use crate::ict_db::Device;

pub fn register(db: Db ,uuid_as_str: &str, pem_public_key: &str) -> Result<String,ICTError> {
    let uuid = Uuid::parse_str(uuid_as_str)?;
    let private_key = RsaPrivateKey::from_pkcs8_pem(pem_public_key)?;
    let secret = Secret::generate_secret();

    let device = Device{id: uuid,
        wrapped_pk: private_key,
        totp_secret: secret.clone()};
    db.add_device(&device)?;
    
    Ok(secret.to_string())
}
