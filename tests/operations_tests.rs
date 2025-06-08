use base64::{engine::general_purpose, Engine as _};
use ict_server::{
    ict_db::Db,
    ict_errors::ICTError,
    ict_operations::{authorize, operate, register},
};
use rand::rngs::OsRng;
use rsa::{
    pkcs1v15::Pkcs1v15Encrypt,
    pkcs8::{EncodePrivateKey, LineEnding},
    RsaPrivateKey, RsaPublicKey,
};
use totp_rs::TOTP;
use uuid::Uuid;

#[test]
fn test_happy_path() -> Result<(), ICTError> {
    let db = Db::new_test_db()?;

    //1 create a fake client
    let id = Uuid::new_v4();

    let mut rng = OsRng;
    let private_key = RsaPrivateKey::new(&mut rng, 2048).expect("failed to generate a key");
    let public_key = RsaPublicKey::from(&private_key);
    let pem_public_key = RsaPrivateKey::to_pkcs8_pem(&private_key, LineEnding::CR)
        .expect("failed to format public key as string");

    println!("pem: {:?}", pem_public_key);

    //2 register client and collect secret
    let secret = register(&db, &id.to_string(), &pem_public_key).expect("failed to register");

    //4 generate totp using device and secret, and encrypt it, mimicing client
    let totp = TOTP::new(
        totp_rs::Algorithm::SHA256, // or SHA256, SHA512
        6,                          // number of digits
        1,                          // step (in 30-second blocks, 1 = 30s)
        30,                         // period (seconds)
        secret.as_bytes().to_vec(),
    )?;
    let token = totp.generate_current()?;
    println!("TOTP token generated {}", token);
    println!("secret generated {}", secret);
    let encrypted_token = public_key.encrypt(&mut OsRng, Pkcs1v15Encrypt, token.as_bytes())?;
    let message = general_purpose::STANDARD.encode(&encrypted_token);

    //5 operate relays, should fail
    match operate(&db, &id.to_string(), &message) {
        Ok(result) => {
            assert!(!result);
        }
        Err(e) => {
            println!("expected err {}", e);
        }
    }

    //6 authorize device
    authorize(&db, &id.to_string()).expect("failed to authorize");

    //7 generate totp using device and secret, and encrypt it, mimicing client
    //optional if authorize is fast enough

    //8 validate totp, should succedd
    assert!(totp
        .check_current(&token)
        .expect("totp internal check failed"));
    //
    assert!(operate(&db, &id.to_string(), &message).expect("failed to operate"));

    Ok(())
}
