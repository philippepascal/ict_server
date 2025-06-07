use rsa::{
    pkcs8::{DecodePrivateKey, EncodePrivateKey},
    RsaPrivateKey,
};
use rusqlite::{params, Connection, Result};
use totp_rs::Secret;
use uuid::Uuid;

use crate::ict_errors::ICTError;

#[derive(Debug)]
pub struct Device {
    pub id: Uuid,
    pub wrapped_pk: RsaPrivateKey,
    pub totp_secret: Secret,
}

impl Device {
    pub fn new(
        id_blob: &Vec<u8>,
        wrapped_pk: &Vec<u8>,
        secret_str: &Vec<u8>,
    ) -> Result<Option<Device>, ICTError> {
        Ok(Some(Device {
            id: Uuid::from_slice(&id_blob)?,
            wrapped_pk: RsaPrivateKey::from_pkcs8_der(&wrapped_pk)?,
            totp_secret: Secret::Raw(secret_str.clone()),
        }))
    }
}

pub struct Db {
    conn: Connection,
}

impl Db {
    pub fn new(db_path: &str) -> Result<Self, ICTError> {
        let conn = Connection::open(db_path)?;
        let db = Db { conn };
        db.init()?;
        Ok(db)
    }

    pub fn new_test_db()-> Result<Self, ICTError> {
        let conn = Connection::open_in_memory()?;
        let db = Db { conn };
        db.init()?;
        Ok(db)
    }

    fn init(&self) -> Result<(), ICTError> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS registered_devices (
                id BLOB PRIMARY KEY,
                wrapped_pk BLOB NOT NULL,
                totp_secret BLOB NOT NULL
            )",
            [],
        )?;
        Ok(())
    }

    pub fn add_device(&self, device: &Device) -> Result<(), ICTError> {
        self.conn.execute(
            "INSERT INTO registered_devices (id, wrapped_pk, totp_secret)
             VALUES (?1, ?2, ?3)",
            params![
                device.id.as_bytes(),
                device.wrapped_pk.to_pkcs8_der()?.as_bytes().to_vec(),
                device.totp_secret.to_bytes()?
            ],
        )?;
        Ok(())
    }

    pub fn get_device(&self, id: Vec<u8>) -> Result<Option<Device>, ICTError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, wrapped_pk, totp_secret FROM registered_devices WHERE id = ?1")?;

        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Device::new(&row.get(0)?, &row.get(1)?, &row.get(2)?)
        } else {
            Ok(None)
        }
    }

    pub fn update_device(&self, device: &Device) -> Result<(), ICTError> {
        self.conn.execute(
            "UPDATE registered_devices SET wrapped_pk = ?2, totp_secret = ?3 WHERE id = ?1",
            params![device.id.as_bytes(), device.wrapped_pk.to_pkcs8_der()?.as_bytes().to_vec(), device.totp_secret.to_bytes()?],
        )?;
        Ok(())
    }

    pub fn delete_device(&self, id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM registered_devices WHERE id = ?1", params![id])?;
        Ok(())
    }
    pub fn count_devices(&self) -> Result<u32> {
        let mut stmt = self
            .conn
            .prepare("SELECT COUNT(*) FROM registered_devices")?;
        let count: u32 = stmt.query_row([], |row| row.get(0))?;
        Ok(count)
    }

    pub fn print_all_devices(&self) -> Result<(), ICTError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, wrapped_pk, totp_secret FROM registered_devices")?;

        let device_iter = stmt.query_map([], |row| {
            Ok(Device::new(&row.get(0)?, &row.get(1)?, &row.get(2)?))
        })?;

        for device in device_iter {
            println!("{:?}", device?);
        }

        Ok(())
    }
}
