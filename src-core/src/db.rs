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
pub use yomitan_writer::{YomitanProgress, YomitanWriter, YomitanZipImportResult, ZipSource};

#[derive(Clone)]
pub(crate) struct Database {
    pub(crate) conn: Arc<Mutex<Connection>>,
    pub(crate) dir: PathBuf,
    yomitan_attached: Arc<AtomicBool>,
    yomitan_glossary_attached: Arc<AtomicBool>,
}

enum DbChild {
    Yomitan,
    YomitanGlossary,
}

const USER_DBFILE: &str = "user.db";
pub(crate) const YOMITAN_DBFILE: &str = "yomitan.db";
pub(crate) const YOMITAN_GLOSSARY_DBFILE: &str = "yomitan-glossary.db";

impl Database {
    pub fn new(db_dir: impl AsRef<Path>) -> Result<Self> {
        let dir = db_dir.as_ref().to_path_buf();
        let conn = Connection::open(dir.join(USER_DBFILE))?;

        conn.execute_batch(
            r"
            PRAGMA jornal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA foreign_keys = ON;
            PRAGMA cache_size = -131072;
            ",
        )?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            dir,
            yomitan_attached: Arc::new(AtomicBool::new(false)),
            yomitan_glossary_attached: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn yomitan(&self) -> Result<yomitan::YomitanDatabase> {
        self.ensure_db_attached(DbChild::Yomitan)?;
        self.ensure_db_attached(DbChild::YomitanGlossary)?;
        Ok(yomitan::YomitanDatabase::new(self.clone()))
    }

    fn ensure_db_attached(&self, db_child: DbChild) -> Result<()> {
        match db_child {
            DbChild::Yomitan => {
                if self.yomitan_attached.load(Ordering::Acquire) {
                    return Ok(());
                }
            }
            DbChild::YomitanGlossary => {
                if self.yomitan_glossary_attached.load(Ordering::Acquire) {
                    return Ok(());
                }
            }
        }

        let conn = self.conn.lock().unwrap();

        let is_attached = match db_child {
            DbChild::Yomitan => self.yomitan_attached.load(Ordering::Relaxed),
            DbChild::YomitanGlossary => self.yomitan_glossary_attached.load(Ordering::Relaxed),
        };

        if !is_attached {
            let dir = self.dir.clone();
            let path = if self.dir.is_absolute() {
                dir
            } else {
                current_dir()?.join(dir)
            };

            let db_file = match db_child {
                DbChild::Yomitan => YOMITAN_DBFILE,
                DbChild::YomitanGlossary => YOMITAN_GLOSSARY_DBFILE,
            };
            let schema_name = match db_child {
                DbChild::Yomitan => "yomitan",
                DbChild::YomitanGlossary => "glossary",
            };

            let path = path.join(db_file).to_string_lossy().replace(r"\", r"/");

            #[cfg(windows)]
            let path = if !path.starts_with("/") {
                format!("/{}", path)
            } else {
                path.to_string()
            };

            let uri = format!("file:{}?mode=ro&immutable=1", path);

            conn.execute(&format!("ATTACH DATABASE ?1 AS {}", schema_name), [uri])?;
            conn.execute_batch(&format!(
                r"
                PRAGMA {}.mmap_size = 3000000000; -- 3_000_000_000
                ",
                schema_name
            ))?;

            match db_child {
                DbChild::Yomitan => self.yomitan_attached.store(true, Ordering::Release),
                DbChild::YomitanGlossary => self
                    .yomitan_glossary_attached
                    .store(true, Ordering::Release),
            };
        }

        Ok(())
    }
}
