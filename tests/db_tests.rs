use ict_server::ict_db::{Db, Device};
use rand::rngs::OsRng;
use rsa::{pkcs1v15::Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};
use std::thread;
use std::time::Duration;
use totp_rs::{Secret, TOTP};
use uuid::Uuid;

#[test]
fn test_db() -> Result<(), Box<dyn std::error::Error>> {
    let db = Db::new_test_db()?;

    let mut rng = OsRng;
    let private_key = RsaPrivateKey::new(&mut rng, 2048).expect("failed to generate a key");
    let public_key = RsaPublicKey::from(&private_key);

    let device = Device {
        id: Uuid::new_v4(),
        wrapped_pk: private_key,
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
    let encrypted_token = public_key.encrypt(&mut OsRng, Pkcs1v15Encrypt, token.as_bytes())?;
    println!("Your encrypted token is : {:?}", encrypted_token);

    let decrypted_bytes = fetched_device
        .wrapped_pk
        .decrypt(Pkcs1v15Encrypt, encrypted_token.as_slice())?;
    let decrypted_token = String::from_utf8(decrypted_bytes).map_err(|_| rsa::Error::Decryption)?;
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
