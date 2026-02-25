use std::{
    collections::HashSet,
    fs::read_dir,
    path::{Path, PathBuf},
};

use rusqlite::Connection;
use serde::Serialize;

use crate::{
    db::{
        Database, YOMITAN_DBFILE, YomitanRow, YomitanWriter, YomitanZipImportProgress,
        YomitanZipImportResult,
    },
    error::CJDicError,
};

#[derive(Serialize, Debug, Clone)]
pub struct LoadYomitanZipDirResult {
    pub new_dicts: Vec<String>,
    pub to_be_removed_dicts: Vec<String>,
}

pub struct AppService {
    db: Database,
}

impl AppService {
    pub fn new<P: AsRef<Path>>(db_dir: P) -> Result<Self, CJDicError> {
        let db = Database::new(db_dir)?;
        Ok(Self { db })
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

    pub fn load_yomitan_zip_dir<LoadCallback, ImportCallback>(
        &self,
        zip_dir: PathBuf,
        lang: &str,
        load_callback: LoadCallback,
        import_callback: ImportCallback,
    ) -> Result<(), CJDicError>
    where
        LoadCallback: Fn(LoadYomitanZipDirResult),
        ImportCallback: Fn(YomitanZipImportProgress),
    {
        let mut dir_zip_list: HashSet<String> = HashSet::new();

        for entry in read_dir(&zip_dir)? {
            let e = entry?;
            let p = e.path();

            if p.extension().and_then(|s| s.to_str()) == Some("zip")
                && let Some(f) = e.file_name().to_str()
            {
                dir_zip_list.insert(f.to_string());
            }
        }

        {
            let conn = Connection::open(self.db.dir.join(YOMITAN_DBFILE))?;
            let mut writer = YomitanWriter::new(conn)?;
            writer.create_schema()?;
        }

        let mut new_dicts: Vec<String> = vec![];
        let mut to_be_removed_dicts: Vec<String> = vec![];

        {
            let conn = Connection::open(self.db.dir.join(YOMITAN_DBFILE))?;
            let mut stmt =
                conn.prepare("SELECT DISTINCT bundle_name FROM dictionaries WHERE lang = ?1")?;
            let mut rows = stmt.query([lang])?;

            let mut db_zip_list: HashSet<String> = HashSet::new();
            while let Some(row) = rows.next()? {
                let b_name: String = row.get(0)?;
                db_zip_list.insert(b_name);
            }

            for z in dir_zip_list.iter() {
                if !db_zip_list.contains(z) {
                    new_dicts.push(z.to_string());
                }
            }

            for z in db_zip_list.iter() {
                if !dir_zip_list.contains(z) {
                    to_be_removed_dicts.push(z.to_string());
                }
            }
        }

        // Import ordering
        new_dicts.sort_by_key(|s| {
            if s.starts_with("[")
                && let Some(end_i) = s.find(']')
            {
                s[(end_i + 1)..].trim_start().to_string()
            } else {
                s.to_string()
            }
        });
        to_be_removed_dicts.sort();

        load_callback(LoadYomitanZipDirResult {
            new_dicts: new_dicts.clone(),
            to_be_removed_dicts: to_be_removed_dicts.clone(),
        });

        {
            let conn = Connection::open(self.db.dir.join(YOMITAN_DBFILE))?;
            for z in to_be_removed_dicts.iter() {
                conn.execute("DELETE FROM dictionaries WHERE bundle_name = ?1", [z])?;
            }
        }

        {
            let conn = Connection::open(self.db.dir.join(YOMITAN_DBFILE))?;
            let mut writer = YomitanWriter::new(conn)?;

            for z in new_dicts.iter() {
                Self::import_yomitan_zip_file(
                    &mut writer,
                    zip_dir.join(z),
                    lang,
                    &import_callback,
                )?;
            }
        }

        Ok(())
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
