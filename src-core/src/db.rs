use std::{path::Path, sync::Mutex};

use rusqlite::{Connection, Result};

use crate::models::Entry;

mod yomitan;
pub use yomitan::{YomitanRow, create_schema, import_bundled_zip_file, search_yomitan};

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn init(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS entries (
                id          INTEGER PRIMARY KEY,
                word        TEXT NOT NULL,
                definition  TEXT NOT NULL
            )",
        )?;
        Ok(())
    }

    pub fn insert_entry(&self, word: &str, definition: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "INSERT INTO entries (word, definition) VALUES (?, ?)",
            [word, definition],
        )?;
        Ok(())
    }

    pub fn fetch_all_entries(&self) -> Result<Vec<Entry>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare("SELECT id, word, definition FROM entries")?;

        let rows = stmt.query_map([], |row| {
            Ok(Entry {
                id: row.get(0)?,
                word: row.get(1)?,
                definition: row.get(2)?,
            })
        })?;

        let mut entries = Vec::new();
        for entry in rows {
            entries.push(entry?);
        }

        Ok(entries)
    }
}
