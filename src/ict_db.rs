// src/db.rs
use rusqlite::{params, Connection, Result};

#[derive(Debug)]
pub struct Device {
    pub id: String,
    pub wrapped_pk: String,
    pub totp_secret: String,
}

pub struct Db {
    conn: Connection,
}

impl Db {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        let db = Db { conn };
        db.init()?;
        Ok(db)
    }

    fn init(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS registered_devices (
                id TEXT PRIMARY KEY,
                wrapped_pk TEXT NOT NULL,
                totp_secret TEXT NOT NULL
            )",
            [],
        )?;
        Ok(())
    }

    pub fn add_device(&self, device: &Device) -> Result<()> {
        self.conn.execute(
            "INSERT INTO registered_devices (id, wrapped_pk, totp_secret)
             VALUES (?1, ?2, ?3)",
            params![device.id, device.wrapped_pk, device.totp_secret],
        )?;
        Ok(())
    }

    pub fn get_device(&self, id: &str) -> Result<Option<Device>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, wrapped_pk, totp_secret FROM registered_devices WHERE id = ?1"
        )?;

        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Device {
                id: row.get(0)?,
                wrapped_pk: row.get(1)?,
                totp_secret: row.get(2)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn update_device(&self, device: &Device) -> Result<()> {
        self.conn.execute(
            "UPDATE registered_devices SET wrapped_pk = ?2, totp_secret = ?3 WHERE id = ?1",
            params![device.id, device.wrapped_pk, device.totp_secret],
        )?;
        Ok(())
    }

    pub fn delete_device(&self, id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM registered_devices WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }
    pub fn count_devices(&self) -> Result<u32> {
    let mut stmt = self.conn.prepare("SELECT COUNT(*) FROM registered_devices")?;
    let count: u32 = stmt.query_row([], |row| row.get(0))?;
    Ok(count)
}

pub fn print_all_devices(&self) -> Result<()> {
    let mut stmt = self.conn.prepare(
        "SELECT id, wrapped_pk, totp_secret FROM registered_devices"
    )?;

    let device_iter = stmt.query_map([], |row| {
        Ok(Device {
            id: row.get(0)?,
            wrapped_pk: row.get(1)?,
            totp_secret: row.get(2)?,
        })
    })?;

    for device in device_iter {
        println!("{:?}", device?);
    }

    Ok(())
}
}