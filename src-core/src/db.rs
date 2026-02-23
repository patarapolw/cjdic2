use std::{
    env::current_dir,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use anyhow::Result;
use rusqlite::Connection;

use crate::models::Entry;

mod yomitan;
pub use yomitan::YomitanRow;

mod yomitan_writer;
pub use yomitan_writer::{YomitanWriter, YomitanZipImportResult};

#[derive(Clone)]
pub(crate) struct Database {
    pub(crate) conn: Arc<Mutex<Connection>>,
    pub(crate) dir: PathBuf,
    yomitan_attached: Arc<AtomicBool>,
}

impl Database {
    pub fn new(db_dir: impl AsRef<Path>) -> Result<Self> {
        let dir = db_dir.as_ref().to_path_buf();
        let conn = Connection::open(dir.join("user.db"))?;

        conn.execute_batch(
            r"
            PRAGMA jornal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA foreign_keys = ON;
            ",
        )?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            dir,
            yomitan_attached: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn yomitan(&self) -> Result<yomitan::YomitanDatabase> {
        self.ensure_yomitan_attached()?;
        Ok(yomitan::YomitanDatabase::new(self.clone()))
    }

    fn ensure_yomitan_attached(&self) -> Result<()> {
        if self.yomitan_attached.load(Ordering::Acquire) {
            return Ok(());
        }

        let conn = self.conn.lock().unwrap();
        if !self.yomitan_attached.load(Ordering::Relaxed) {
            let dir = self.dir.clone();
            let path = if self.dir.is_absolute() {
                dir
            } else {
                current_dir()?.join(dir)
            };
            let path = path
                .join("yomitan.db")
                .to_string_lossy()
                .replace(r"\", r"/");

            #[cfg(windows)]
            let path = if !path.starts_with("/") {
                format!("/{}", path)
            } else {
                path.to_string()
            };

            let uri = format!("file:{}?mode=ro&immutable=1", path);

            conn.execute("ATTACH DATABASE ?1 AS yomitan", [uri])?;
            self.yomitan_attached.store(true, Ordering::Release);
        }

        Ok(())
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
