use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::{
    db::{
        Database, YOMITAN_DBFILE, YomitanRow, YomitanWriter, YomitanZipImportProgress,
        YomitanZipImportResult,
    },
    error::CJDicError,
};

pub struct AppService {
    db: Database,
}

impl AppService {
    pub fn new<P: AsRef<Path>>(db_dir: P) -> Result<Self, CJDicError> {
        let db = Database::new(db_dir)?;
        Ok(Self { db })
    }

    pub fn is_yomitan_setup_yet(&self) -> Result<bool, CJDicError> {
        Ok(self.db.is_yomitan_dbfile_init()?)
    }

    pub fn search_yomitan(
        &self,
        q_term: &str,
        q_reading: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<YomitanRow>, CJDicError> {
        Ok(self
            .db
            .yomitan()?
            .search_yomitan(q_term, q_reading, limit, offset)?)
    }

    pub fn get_yomitan_writer(&self) -> Result<YomitanWriter, CJDicError> {
        let conn = Connection::open(self.db.dir.join(YOMITAN_DBFILE))?;
        let mut writer = YomitanWriter::new(conn)?;
        writer.create_schema()?;
        Ok(writer)
    }

    pub fn import_yomitan_zip_file<Callback>(
        writer: &mut YomitanWriter,
        zip_file: PathBuf,
        lang: &str,
        progress_callback: Callback,
    ) -> Result<YomitanZipImportResult, CJDicError>
    where
        Callback: Fn(YomitanZipImportProgress),
    {
        Ok(writer.import_dictionary_zip_file(zip_file, lang, progress_callback)?)
    }

    pub fn remove_yomitan_dictionary(
        writer: &mut YomitanWriter,
        title: &str,
    ) -> Result<(), CJDicError> {
        Ok(writer.drop_dictionary(title)?)
    }
}
