use rusqlite::{params, Connection, Result};
use sha2::{Sha256, Digest};
use std::sync::Mutex;

pub struct LocalBrain {
    conn: Mutex<Connection>,
}

impl LocalBrain {
    pub fn init(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS mesh_dag (
                sequence INTEGER PRIMARY KEY AUTOINCREMENT,
                id TEXT UNIQUE NOT NULL,
                parent_id TEXT,
                encrypted_payload BLOB NOT NULL,
                timestamp INTEGER NOT NULL
            )",
            [],
        )?;
        conn.execute(
            "INSERT OR IGNORE INTO mesh_dag (id, parent_id, encrypted_payload, timestamp) 
             VALUES ('GENESIS', NULL, x'', 0)",
            [],
        )?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    pub fn append_to_dag(&self, encrypted_payload: &[u8]) -> Result<String> {
        let conn = self.conn.lock().unwrap();
        
        let parent_id: String = conn.query_row(
            "SELECT id FROM mesh_dag ORDER BY sequence DESC LIMIT 1",
            [],
            |row| row.get(0),
        )?;
        let mut hasher = Sha256::new();
        hasher.update(encrypted_payload);
        let id = hex::encode(hasher.finalize());
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64; 
        
        conn.execute(
            "INSERT INTO mesh_dag (id, parent_id, encrypted_payload, timestamp) VALUES (?1, ?2, ?3, ?4)",
            params![id, parent_id, encrypted_payload, now],
        )?;
        Ok(id)
    }
}
