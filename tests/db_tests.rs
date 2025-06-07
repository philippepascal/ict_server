use ict_server::ict_db::{Db,Device};
use uuid::Uuid;
use totp_rs::Secret;
use rsa::{RsaPrivateKey, RsaPublicKey};
use rand::rngs::OsRng;

#[test]
fn test_db() -> Result<(),Box<dyn std::error::Error>> {
    let db = Db::new_test_db()?;

    let mut rng = OsRng;
    let private_key = RsaPrivateKey::new(&mut rng, 2048)
        .expect("failed to generate a key");
    let _public_key = RsaPublicKey::from(&private_key);

    let device = Device {
        id: Uuid::new_v4(),
        wrapped_pk: private_key,
        totp_secret: Secret::generate_secret(),
    };

    db.add_device(&device).unwrap();
    db.print_all_devices().unwrap();
    let loaded = db.get_device(device.id.as_bytes().to_vec()).unwrap();
    // println!("Loaded device: {:?}", loaded.unwrap());
    assert!(loaded.is_some());
    Ok(())
}