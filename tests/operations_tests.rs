use base64::{engine::general_purpose, Engine as _};
use ict_server::{
    ict_db::Db,
    ict_errors::ICTError,
    ict_operations::{associate_relay, authorize, operate, register},
    ict_operations::OperationMessage,
};
use rand::rngs::OsRng;
use rsa::{
    pkcs8::{EncodePublicKey, LineEnding},
    RsaPrivateKey, RsaPublicKey,
};
use rsa::{pkcs1v15::SigningKey, signature::Signer};
use sha2::Sha256;
use totp_rs::{Secret, TOTP};
use uuid::Uuid;

#[test]
fn test_happy_path() -> Result<(), ICTError> {
    let _ = env_logger::Builder::from_default_env()
        .format_timestamp_secs()
        .is_test(true)
        .try_init();
    let db = Db::new_test_db()?;

    //1 create a fake client
    let id = Uuid::new_v4();

    let mut rng = OsRng;
    let private_key = RsaPrivateKey::new(&mut rng, 2048).expect("failed to generate a key");
    let public_key = RsaPublicKey::from(&private_key);
    let pem_public_key = RsaPublicKey::to_public_key_pem(&public_key, LineEnding::CR)
        .expect("failed to format public key as string");

    println!("pem: {:?}", pem_public_key);

    //2 register client and collect secret
    let secret_string =
        register(&db, &id.to_string(), &pem_public_key).expect("failed to register");
    println!("secret generated (encoded) {}", &secret_string);
    let secret = Secret::Encoded(secret_string);

    //4 generate totp using device and secret, and encrypt it, mimicing client
    let totp = TOTP::new(
        totp_rs::Algorithm::SHA256, // or SHA256, SHA512
        6,                          // number of digits
        1,                          // step (in 30-second blocks, 1 = 30s)
        30,                         // period (seconds)
        secret.to_bytes().unwrap(),
    )?;
    let token = totp.generate_current()?;
    println!("TOTP token generated {}", token);

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

    //5 operate relays, should fail
    match operate(&db, &id.to_string(), &message, &signature_base64) {
        Ok(result) => {
            assert!(!result);
        }
        Err(e) => {
            println!("expected err {}", e);
        }
    }

    //5 add relays
    associate_relay(&db, &id.to_string(), &2)?;
    associate_relay(&db, &id.to_string(), &5)?;

    //6 authorize device
    authorize(&db, &id.to_string()).expect("failed to authorize");

    //7 generate totp using device and secret, and encrypt it, mimicing client
    //optional if authorize is fast enough

    //8 validate totp, should succedd
    assert!(totp
        .check_current(&token)
        .expect("totp internal check failed")); //internal check
                                                //9. actual successful call to operate!!
    assert!(operate(&db, &id.to_string(), &message, &signature_base64).expect("failed to operate"));

    Ok(())
}
