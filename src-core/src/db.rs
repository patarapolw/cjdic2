use std::{
    env::current_dir,
    path::{Path, PathBuf},
    sync::{
        Arc, LazyLock, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use anyhow::{Ok, Result};
use enum_map::EnumMap;
use rusqlite::Connection;

mod yomitan;
pub use yomitan::YomitanRow;

mod yomitan_writer;
pub use yomitan_writer::{YomitanProgress, YomitanWriter, YomitanZipImportResult, ZipSource};

#[derive(Clone)]
pub(crate) struct Database {
    pub(crate) conn: Arc<Mutex<Connection>>,
    pub(crate) dir: PathBuf,
    is_db_attached: EnumMap<DbChild, Arc<AtomicBool>>,
}

#[derive(Debug, Enum, Clone, Copy)]
pub(crate) enum DbChild {
    Yomitan,
    YomitanGlossary,
    Reading,
}

const USER_DBFILE: &str = "user.db";

pub(crate) const DBFILE: LazyLock<EnumMap<DbChild, &str>> = LazyLock::new(|| {
    enum_map! {
        DbChild::Yomitan => "yomitan.db",
        DbChild::YomitanGlossary => "yomitan-glossary.db",
        DbChild::Reading => "reading.db"
    }
});
pub(crate) const DBSCHEMA: LazyLock<EnumMap<DbChild, &str>> = LazyLock::new(|| {
    enum_map! {
        DbChild::Yomitan => "yomitan",
        DbChild::YomitanGlossary => "glossary",
        DbChild::Reading => "reading"
    }
});

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
            is_db_attached: enum_map! {
                DbChild::Yomitan => Arc::new(AtomicBool::new(false)),
                DbChild::YomitanGlossary => Arc::new(AtomicBool::new(false)),
                DbChild::Reading => Arc::new(AtomicBool::new(false)),
            },
        })
    }

    pub fn yomitan(&self) -> Result<yomitan::YomitanDatabase> {
        self.ensure_db_attached(DbChild::Yomitan)?;
        self.ensure_db_attached(DbChild::YomitanGlossary)?;
        self.ensure_db_attached(DbChild::Reading)?;
        Ok(yomitan::YomitanDatabase::new(self.clone()))
    }

    fn ensure_db_attached(&self, db_child: DbChild) -> Result<()> {
        if self.is_db_attached[db_child].load(Ordering::Acquire) {
            return Ok(());
        }

        let conn = self.conn.lock().unwrap();
        let is_attached = self.is_db_attached[db_child].load(Ordering::Relaxed);

        if !is_attached {
            let dir = self.dir.clone();
            let path = if self.dir.is_absolute() {
                dir
            } else {
                current_dir()?.join(dir)
            };

            let mode = match db_child {
                DbChild::Yomitan | DbChild::YomitanGlossary => "?mode=ro&immutable=1",
                _ => "",
            };

            let path = path
                .join(DBFILE[db_child])
                .to_string_lossy()
                .replace(r"\", r"/");

            #[cfg(windows)]
            let path = if !path.starts_with("/") {
                format!("/{}", path)
            } else {
                path.to_string()
            };

            let uri = format!("file:{}{}", path, mode);

            conn.execute(
                &format!("ATTACH DATABASE ?1 AS {}", DBSCHEMA[db_child]),
                [uri],
            )?;
            conn.execute_batch(&format!(
                r"
                PRAGMA {}.mmap_size = 3000000000; -- 3_000_000_000
                ",
                DBSCHEMA[db_child]
            ))?;

            self.is_db_attached[db_child].store(true, Ordering::Release);
        }

        Ok(())
    }
}
