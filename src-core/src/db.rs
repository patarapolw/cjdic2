use std::{
    env::current_dir,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use anyhow::{Ok, Result};
use rusqlite::Connection;

mod yomitan;
pub use yomitan::YomitanRow;

mod yomitan_writer;
pub use yomitan_writer::{YomitanWriter, YomitanZipImportProgress, YomitanZipImportResult};

#[derive(Clone)]
pub(crate) struct Database {
    pub(crate) conn: Arc<Mutex<Connection>>,
    pub(crate) dir: PathBuf,
    yomitan_attached: Arc<AtomicBool>,
}

const USER_DBFILE: &str = "user.db";
pub(crate) const YOMITAN_DBFILE: &str = "yomitan.db";

impl Database {
    pub fn new(db_dir: impl AsRef<Path>) -> Result<Self> {
        let dir = db_dir.as_ref().to_path_buf();
        let conn = Connection::open(dir.join(USER_DBFILE))?;

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
                .join(YOMITAN_DBFILE)
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
}
