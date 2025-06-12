use rsa::{
    pkcs8::{DecodePublicKey, EncodePublicKey},
    RsaPublicKey,
};
use rusqlite::{params, Connection, Result};
use totp_rs::Secret;
use uuid::Uuid;

use crate::ict_errors::ICTError;

#[derive(Debug)]
pub struct Device {
    pub id: Uuid,
    pub wrapped_pk: RsaPublicKey,
    pub totp_secret: Secret,
    pub authorized: u8,
}

#[derive(Debug)]
pub struct Relay {
    pub device_id: Uuid,
    pub relay_id: u8,
}

impl Device {
    pub fn new(
        id_blob: &Vec<u8>,
        wrapped_pk: &Vec<u8>,
        secret: &Vec<u8>,
        authorized: u8,
    ) -> Result<Device, ICTError> {
        Ok(Device {
            id: Uuid::from_slice(&id_blob)?,
            wrapped_pk: RsaPublicKey::from_public_key_der(&wrapped_pk)?,
            totp_secret: Secret::Raw(secret.clone()),
            authorized: authorized,
        })
    }
}
pub struct Db {
    pub path: Option<String>,
    conn: Connection,
}

impl Db {

    pub fn newg(db_path: Option<String>) -> Result<Self, ICTError> {
        match db_path {
            Some(path) => Self::new(&path),
            None => Self::new_test_db(),
        }
    }

    pub fn new(db_path: &str) -> Result<Self, ICTError> {
        let conn = Connection::open(db_path)?;
        let db = Db { path: Some(db_path.to_string()), conn };
        db.init()?;
        Ok(db)
    }

    pub fn new_test_db() -> Result<Self, ICTError> {
        let conn = Connection::open_in_memory()?;
        let db = Db { path: None, conn };
        db.init()?;
        Ok(db)
    }

    fn init(&self) -> Result<(), ICTError> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS registered_devices (
                id BLOB PRIMARY KEY,
                wrapped_pk BLOB NOT NULL,
                totp_secret BLOB NOT NULL,
                authorized INTEGER NOT NULL
            )",
            [],
        )?;
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS relays (
                device_id BLOB NOT NULL,
                relay_id INTEGER NOT NULL,
                FOREIGN KEY(device_id) REFERENCES registered_devices(id))",
            [],
        )?;
        Ok(())
    }

    pub fn add_device(&self, device: &Device) -> Result<(), ICTError> {
        self.conn.execute(
            "INSERT INTO registered_devices (id, wrapped_pk, totp_secret, authorized)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                device.id.as_bytes(),
                device.wrapped_pk.to_public_key_der()?.as_bytes().to_vec(),
                device.totp_secret.to_bytes()?,
                device.authorized,
            ],
        )?;
        Ok(())
    }

    pub fn add_relay(&self, device_id: Uuid, relay_id: u8) -> Result<(), ICTError> {
        self.conn.execute(
            "INSERT INTO relays (device_id, relay_id) VALUES (?1, ?2)",
            params![device_id.as_bytes(), relay_id],
        )?;
        Ok(())
    }

    pub fn get_device(&self, id: Uuid) -> Result<Option<Device>, ICTError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, wrapped_pk, totp_secret, authorized FROM registered_devices WHERE id = ?1",
        )?;

        let mut rows = stmt.query(params![id.as_bytes()])?;
        if let Some(row) = rows.next()? {
            Device::new(&row.get(0)?, &row.get(1)?, &row.get(2)?, row.get(3)?).map(Some)
        } else {
            Ok(None)
        }
    }

    pub fn get_devices(&self) -> Result<Vec<Device>, ICTError> {
        let mut stmt = self.conn.prepare("SELECT * FROM registered_devices")?;
        let rows = stmt.query_map([], |row| {
            Device::new(&row.get(0)?, &row.get(1)?, &row.get(2)?, row.get(3)?)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
        })?;

        let devices = rows
            .filter_map(|res| match res {
                Ok(dev) => Some(Ok(dev)),
                Err(e) => Some(Err(e.into())),
            })
            .collect::<Result<Vec<Device>, ICTError>>()?;

        Ok(devices)
    }

    pub fn get_relays(&self, device_id: Uuid) -> Result<Vec<u8>, ICTError> {
        let mut stmt = self
            .conn
            .prepare("SELECT relay_id FROM relays WHERE device_id = ?1")?;
        let rows = stmt.query_map(params![device_id.as_bytes()], |row| row.get(0))?;
        Ok(rows.collect::<Result<Vec<u8>, _>>()?)
    }

    pub fn update_device(&self, device: &Device) -> Result<(), ICTError> {
        self.conn.execute(
            "UPDATE registered_devices SET wrapped_pk = ?2, totp_secret = ?3, authorized = ?4 WHERE id = ?1",
            params![device.id.as_bytes(), device.wrapped_pk.to_public_key_der()?.as_bytes().to_vec(), device.totp_secret.to_bytes()?, device.authorized],
        )?;
        Ok(())
    }

    pub fn set_authorization_on_device(&self, id: Uuid, auth: u8) -> Result<(), ICTError> {
        self.conn.execute(
            "UPDATE registered_devices SET authorized = ?2 WHERE id = ?1",
            params![id.as_bytes(), auth],
        )?;
        Ok(())
    }

    pub fn delete_device(&self, id: Uuid) -> Result<()> {
        self.conn.execute(
            "DELETE FROM registered_devices WHERE id = ?1",
            params![id.as_bytes()],
        )?;
        Ok(())
    }

    pub fn remove_relays(&self, device_id: Uuid) -> Result<()> {
        self.conn.execute(
            "DELETE FROM relays WHERE device_id = ?1",
            params![device_id.as_bytes()],
        )?;
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
            .prepare("SELECT id, wrapped_pk, totp_secret, authorized FROM registered_devices")?;

        let device_iter = stmt.query_map([], |row| {
            Ok(Device::new(
                &row.get(0)?,
                &row.get(1)?,
                &row.get(2)?,
                row.get(3)?,
            ))
        })?;

        for device in device_iter {
            println!("{:?}", device?);
        }

        Ok(())
    }
}
