use base64::{engine::general_purpose, Engine as _};
use rsa::{pkcs1v15::Pkcs1v15Encrypt, pkcs8::DecodePrivateKey, RsaPrivateKey};
use totp_rs::{Secret, TOTP};
use uuid::Uuid;
use log::{info};

use crate::ict_db::Db;
use crate::ict_db::Device;
use crate::ict_errors::ICTError;

pub fn register(db: &Db, uuid_as_str: &str, pem_public_key: &str) -> Result<String, ICTError> {
    let uuid = Uuid::parse_str(uuid_as_str)?;
    let private_key = RsaPrivateKey::from_pkcs8_pem(pem_public_key)?;
    let secret = Secret::generate_secret();

    let device = Device {
        id: uuid,
        wrapped_pk: private_key,
        totp_secret: secret.clone(),
        authorized: 0,
    };
    db.add_device(&device)?;

    Ok(device.totp_secret.to_encoded().to_string())
}

pub fn authorize(db: &Db, uuid_as_str: &str) -> Result<(), ICTError> {
    set_auth(db, uuid_as_str, 1)
}

pub fn unauthorize(db: &Db, uuid_as_str: &str) -> Result<(), ICTError> {
    set_auth(db, uuid_as_str, 0)
}

pub fn set_auth(db: &Db, uuid_as_str: &str, auth_value: u8) -> Result<(), ICTError> {
    let uuid = Uuid::parse_str(uuid_as_str)?;

    db.set_authorization_on_device(uuid, auth_value)?;

    Ok(())
}

pub fn delete_device(db: &Db, uuid_as_str: &str) -> Result<(), ICTError> {
    let uuid = Uuid::parse_str(uuid_as_str)?;

    db.delete_device(uuid)?;

    Ok(())
}

pub fn operate(db: &Db, uuid_as_str: &str, message: &str) -> Result<bool, ICTError> {
    let uuid = Uuid::parse_str(uuid_as_str)?;
    let device = db.get_device(uuid)?.ok_or(ICTError::Custom(
        "No device with that uuid found".to_string(),
    ))?;
    if device.authorized != 1 {
        return Err(ICTError::Custom(
            "Will not operate a device/client that is not authorized".to_string(),
        ));
    }
    let encrypted_bytes = general_purpose::STANDARD.decode(&message)?;
    let decrypted_bytes = device
        .wrapped_pk
        .decrypt(Pkcs1v15Encrypt, encrypted_bytes.as_slice())?;
    let decrypted_token = String::from_utf8(decrypted_bytes).map_err(|_| rsa::Error::Decryption)?;
    let totp = TOTP::new(
        totp_rs::Algorithm::SHA256, // or SHA256, SHA512
        6,                          // number of digits
        1,                          // step (in 30-second blocks, 1 = 30s)
        30,                         // period (seconds)
        device.totp_secret.to_bytes().unwrap(),
    )?;

    if totp.check_current(&decrypted_token)? {
        // here perform the relay logic (close the circuit for limit time)
        let relays = db.get_relays(device.id)?;
        relays.iter().for_each(|relay| {
            info!("operating relay {} for uuid {}",relay,uuid_as_str);
        });
        Ok(true)
    } else {
        Err(ICTError::Custom("TOTP token is not valid".to_string()))
    }
}

pub fn list_clients(db: &Db) -> Result<(),ICTError> {
    info!("Listing registered clients:");
    let devices = db.get_devices()?;
    for d in devices {
        info!("Client {:?}",d);
        let relays = db.get_relays(d.id)?;
        for r in relays {
            info!("   has relay {}",r);
        }
    }
    Ok(())
}

pub fn describe_client(db: &Db, uuid_as_str: &str) -> Result<(),ICTError>{
    info!("Describing registered client {}:",uuid_as_str);
    let uuid = Uuid::parse_str(uuid_as_str)?;
    let device = db.get_device(uuid)?;
    info!("Client {:?}",device);
    for relay in db.get_relays(uuid)? {
        info!("    has relay {}",relay);
    }
    Ok(())
}

pub fn associate_relay(db: &Db, uuid_as_str: &str, relay: &u8) -> Result<(),ICTError>{
    let uuid = Uuid::parse_str(uuid_as_str)?;
    db.add_relay(uuid, *relay)?;
    Ok(())
}

pub fn clear_relays(db: &Db, uuid_as_str: &str) -> Result<(),ICTError> {
    let uuid = Uuid::parse_str(uuid_as_str)?;
    db.remove_relays(uuid)?;
    Ok(())
}