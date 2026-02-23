use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::{
    db::{Database, YomitanRow, YomitanWriter, YomitanZipImportResult},
    error::CJDicError,
    models::Entry,
};

pub struct AppService {
    db: Database,
}

impl AppService {
    pub fn new<P: AsRef<Path>>(db_dir: P) -> Result<Self, CJDicError> {
        let db = Database::new(db_dir)?;
        db.init()?;
        Ok(Self { db })
    }

    pub fn add_entry(&self, word: &str, definition: &str) -> Result<(), CJDicError> {
        self.db.insert_entry(word, definition)?;
        Ok(())
    }

    pub fn list_entries(&self) -> Result<Vec<Entry>, CJDicError> {
        Ok(self.db.fetch_all_entries()?)
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
        let conn = Connection::open(self.db.dir.join("yomitan.db"))?;
        let writer = YomitanWriter::new(conn)?;
        writer.create_schema()?;
        Ok(writer)
    }

    pub fn import_yomitan_zip_file(
        writer: &mut YomitanWriter,
        zip_file: PathBuf,
        lang: &str,
    ) -> Result<YomitanZipImportResult, CJDicError> {
        Ok(writer.import_dictionary_zip_file(zip_file, lang)?)
    }

    pub fn remove_yomitan_dictionary(
        writer: &mut YomitanWriter,
        title: &str,
    ) -> Result<(), CJDicError> {
        Ok(writer.drop_dictionary(title)?)
    }
}
