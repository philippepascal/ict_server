
use base64::{engine::general_purpose, Engine as _};
use rsa::pkcs1v15::Signature;
use rsa::pkcs8::DecodePublicKey;
use rsa::{pkcs1v15::Pkcs1v15Encrypt, RsaPublicKey};
use totp_rs::{Secret, TOTP};
use uuid::Uuid;
use log::{info};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use rsa::pkcs1v15::VerifyingKey;
use rsa::signature::Verifier;
use rsa::sha2::Sha256;

use crate::ict_db::Db;
use crate::ict_db::Device;
use crate::ict_errors::ICTError;

#[derive(Deserialize,Serialize)]
pub struct OperationMessage {
    pub token: String,
    pub _salt: String,
}

pub fn register(db: &Db, uuid_as_str: &str, pem_public_key: &str) -> Result<String, ICTError> {
    let uuid = Uuid::parse_str(uuid_as_str)?;
    let public_key = RsaPublicKey::from_public_key_pem(pem_public_key)?;
    let secret = Secret::generate_secret();

    let device = Device {
        id: uuid,
        wrapped_pk: public_key,
        totp_secret: secret.clone(),
        authorized: 0,
    };
    db.add_device(&device)?;

    // let encrypted_secret = device.wrapped_pk.encrypt(&mut OsRng, Pkcs1v15Encrypt, &device.totp_secret.to_bytes()?)?;

    // let encrypted_secret = device.wrapped_pk.encrypt(&mut OsRng, Pkcs1v15Encrypt, &device.totp_secret.to_encoded().to_bytes()?)?;

    println!("secret {}",&device.totp_secret.to_encoded().to_string());
    let encrypted_secret = device.wrapped_pk.encrypt(&mut OsRng, Pkcs1v15Encrypt, &device.totp_secret.to_encoded().to_string().as_bytes())?;

    Ok(general_purpose::STANDARD.encode(&encrypted_secret))

    // Ok(String::from_utf8(encrypted_secret)?)
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

pub fn operate(db: &Db, uuid_as_str: &str, message: &str, signature: &str) -> Result<bool, ICTError> {
    let uuid = Uuid::parse_str(uuid_as_str)?;
    let device = db.get_device(uuid)?.ok_or(ICTError::Custom(
        "No device with that uuid found".to_string(),
    ))?;
    if device.authorized != 1 {
        return Err(ICTError::Custom(
            "Will not operate a device/client that is not authorized".to_string(),
        ));
    }
    
    // check signature
    let verifying_key = VerifyingKey::<Sha256>::new(device.wrapped_pk);
    let signature_bytes = general_purpose::STANDARD.decode(signature)
        .map_err(|_| ICTError::Custom("Failed to decode base64 signature".into()))?;

    match verifying_key.verify(&message.as_bytes(), &Signature::try_from(signature_bytes.as_slice())?) {
        Ok(()) => (),
        Err(_e) => return Result::Err(ICTError::Custom("Message signature verification failed".to_string())),
    }

    // unpack json message {token,salt}
    let parsed: OperationMessage = serde_json::from_str(message)
        .map_err(|_| ICTError::Custom("Failed to parse JSON message".into()))?;
    let decrypted_token = parsed.token;

    let algo = match db.totp_sha.as_str() {
        "sha1" => totp_rs::Algorithm::SHA1,
        "sha512" => totp_rs::Algorithm::SHA512,
        _ => totp_rs::Algorithm::SHA256,
    };

    let totp = TOTP::new(
        algo,
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
