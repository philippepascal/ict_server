use ict_server::{
    ict_db::{Db, Device},
    ict_errors::ICTError,
    ict_operations::OperationMessage,
};
use rand::rngs::OsRng;
use rsa::{ RsaPrivateKey, RsaPublicKey};
use std::{thread};
use std::time::Duration;
use totp_rs::{Secret, TOTP};
use uuid::Uuid;
use rsa::{pkcs1v15::SigningKey, signature::Signer,pkcs1v15::VerifyingKey,pkcs1v15::Signature,signature::Verifier};
use sha2::Sha256;
use base64::{engine::general_purpose, Engine as _};


#[test]
fn test_device() -> Result<(), ICTError> {
    let _ = env_logger::Builder::from_default_env()
        .format_timestamp_secs()
        .is_test(true)
        .try_init();
    let db = Db::new_test_db()?;

    let mut rng = OsRng;
    let private_key = RsaPrivateKey::new(&mut rng, 2048).expect("failed to generate a key");
    let public_key = RsaPublicKey::from(&private_key);

    let device = Device {
        id: Uuid::new_v4(),
        wrapped_pk: public_key,
        totp_secret: Secret::generate_secret(),
        authorized: 0,
    };

    db.add_device(&device).unwrap();
    db.print_all_devices().unwrap();
    let loaded = db.get_device(device.id).unwrap();
    // println!("Loaded device: {:?}", loaded.unwrap());
    assert!(loaded.is_some());
    let fetched_device = loaded.unwrap();
    assert_eq!(fetched_device.id, device.id);
    assert_eq!(fetched_device.totp_secret, device.totp_secret);
    assert_eq!(fetched_device.wrapped_pk, device.wrapped_pk);

    let totp = TOTP::new(
        totp_rs::Algorithm::SHA256, // or SHA256, SHA512
        6,                          // number of digits
        1,                          // step (in 30-second blocks, 1 = 30s)
        30,                         // period (seconds)
        device.totp_secret.to_bytes().unwrap(),
    )?;

    let token = totp.generate_current()?;
    println!("Your TOTP token is: {}", token);

    let op_message = OperationMessage {
        token: token.clone(),
        _salt: "asdf".to_string(),
    };
    let message = serde_json::to_string(&op_message).unwrap();

    // Step 3: Sign the message
    let signing_key = SigningKey::<Sha256>::new(private_key);
    let signature = signing_key.sign(message.as_bytes());

    // Step 4: Print base64-encoded signature
    let signature_base64 = general_purpose::STANDARD.encode(signature.to_string());
    println!("Signature (base64): {}", signature_base64);


    let verifying_key = VerifyingKey::<Sha256>::new(device.wrapped_pk);
    let signature_bytes = general_purpose::STANDARD.decode(signature_base64)
        .map_err(|_| ICTError::Custom("Failed to decode base64 signature".into()))?;
    verifying_key.verify(&message.as_bytes(), &Signature::try_from(signature_bytes.as_slice())?).unwrap();

    // unpack json message {token,salt}
    let parsed: OperationMessage = serde_json::from_str(&message)
        .map_err(|_| ICTError::Custom("Failed to parse JSON message".into()))?;
    let decrypted_token = parsed.token;

    println!("Your decrypted token is : {}", decrypted_token);

    assert_eq!(decrypted_token, token);

    let totp2 = TOTP::new(
        totp_rs::Algorithm::SHA256, // or SHA256, SHA512
        6,                          // number of digits
        1,                          // step (in 30-second blocks, 1 = 30s)
        30,                         // period (seconds)
        fetched_device.totp_secret.to_bytes().unwrap(),
    )?;

    assert!(totp2.check_current(&decrypted_token).unwrap());
    thread::sleep(Duration::from_secs(60));
    assert!(!totp2.check_current(&decrypted_token).unwrap());

    Ok(())
}

#[test]
fn test_relays() -> Result<(), ICTError> {
    let _ = env_logger::Builder::from_default_env()
        .format_timestamp_secs()
        .is_test(true)
        .try_init();
    let db = Db::new_test_db()?;

    let mut rng = OsRng;
    let private_key = RsaPrivateKey::new(&mut rng, 2048).expect("failed to generate a key");
    let public_key = RsaPublicKey::from(&private_key);

    let device = Device {
        id: Uuid::new_v4(),
        wrapped_pk: public_key,
        totp_secret: Secret::generate_secret(),
        authorized: 0,
    };

    db.add_device(&device).unwrap();
    db.print_all_devices().unwrap();

    assert!(db.get_relays(device.id)?.is_empty());
    db.add_relay(device.id, 1)?;
    assert_eq!(db.get_relays(device.id)?.len(), 1);
    for i in db.get_relays(device.id)? {
        println!("Device {} has relay {}", device.id, i);
        assert_eq!(i, 1);
    }
    db.add_relay(device.id, 4)?;
    assert_eq!(db.get_relays(device.id)?.len(), 2);
    for i in db.get_relays(device.id)? {
        println!("Device {} has relay {}", device.id, i);
    }
    assert_eq!(db.get_relays(device.id)?[1], 4);
    db.remove_relays(device.id)?;
    assert!(db.get_relays(device.id)?.is_empty());
    Ok(())
}
