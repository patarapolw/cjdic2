use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use rusqlite::{Connection, params_from_iter};
use serde::{Deserialize, Serialize};

use crate::{
    Timer, ZipSource,
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

#[derive(Deserialize)]
#[serde(untagged)]
pub enum SqlParam {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
    Bool(bool),
}

pub struct AppService {
    db: Database,
}

impl AppService {
    pub fn new<P: AsRef<Path>>(db_dir: P) -> Result<Self, CJDicError> {
        let db = Database::new(db_dir)?;
        Ok(Self { db })
    }

    pub fn execute_sql(
        &self,
        sql: String,
        params: Vec<SqlParam>,
    ) -> Result<serde_json::Value, CJDicError> {
        let _timer = Timer::new(sql.to_string());

        let conn = self.db.conn.lock().unwrap();
        let converted_params: Vec<_> = params
            .into_iter()
            .map(|p| match p {
                SqlParam::Null => rusqlite::types::Value::Null,
                SqlParam::Integer(i) => rusqlite::types::Value::Integer(i),
                SqlParam::Real(n) => rusqlite::types::Value::Real(n),
                SqlParam::Text(s) => rusqlite::types::Value::Text(s),
                SqlParam::Bool(b) => rusqlite::types::Value::Integer(b as i64),
            })
            .collect();

        let mut stmt = conn.prepare(&sql)?;

        if stmt.column_count() == 0 {
            let affected = stmt.execute(params_from_iter(converted_params))?;
            return Ok(serde_json::Value::Number(affected.into()));
        }

        let column_names: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();
        let rows = stmt.query_map(params_from_iter(converted_params), |row| {
            let mut obj = serde_json::Map::new();

            for (i, name) in column_names.iter().enumerate() {
                let value = match row.get_ref(i)? {
                    rusqlite::types::ValueRef::Null => serde_json::Value::Null,
                    rusqlite::types::ValueRef::Integer(i) => serde_json::Value::Number(i.into()),
                    // Boolean are also rendered as an integer.
                    rusqlite::types::ValueRef::Real(n) => serde_json::Number::from_f64(n)
                        .map(serde_json::Value::Number)
                        .unwrap_or(serde_json::Value::Null),
                    rusqlite::types::ValueRef::Text(t) => {
                        serde_json::Value::String(String::from_utf8_lossy(t).into())
                    }
                    rusqlite::types::ValueRef::Blob(_) => serde_json::Value::Null,
                };
                obj.insert(name.clone(), value);
            }
            return Ok(serde_json::Value::Object(obj));
        })?;

        let result: Result<Vec<_>, _> = rows.collect();
        Ok(serde_json::Value::Array(result?))
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

    pub fn cleanup_yomitan_writer(&self) -> Result<YomitanWriter, CJDicError> {
        let conn = Connection::open(self.db.dir.join(YOMITAN_DBFILE))?;
        let writer = YomitanWriter::new(conn)?;
        writer.cleanup()?;
        Ok(writer)
    }

    pub fn load_yomitan_zip_dir<Z, LoadCallback, ImportCallback>(
        &self,
        zip_dir: Vec<Z>,
        lang: &str,
        load_callback: LoadCallback,
        import_callback: ImportCallback,
    ) -> Result<LoadYomitanZipDirResult, CJDicError>
    where
        Z: ZipSource,
        LoadCallback: Fn(LoadYomitanZipDirResult),
        ImportCallback: Fn(YomitanZipImportProgress),
    {
        let mut dir_zip_map: HashMap<String, Z> = HashMap::new();

        for entry in zip_dir {
            dir_zip_map.insert(entry.file_name().to_string(), entry);
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

            for z in dir_zip_map.keys() {
                if !db_zip_list.contains(&z.to_string()) {
                    new_dicts.push(z.to_string());
                }
            }

            for z in db_zip_list.iter() {
                if !dir_zip_map.contains_key(z) {
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
            for z in to_be_removed_dicts.clone().iter() {
                conn.execute("DELETE FROM dictionaries WHERE bundle_name = ?1", [z])?;
            }
        }

        {
            let conn = Connection::open(self.db.dir.join(YOMITAN_DBFILE))?;
            let mut writer = YomitanWriter::new(conn)?;

            for z in new_dicts.clone().iter() {
                if let Some(zip_file) = dir_zip_map.get(z) {
                    Self::import_yomitan_zip_file(&mut writer, zip_file, lang, &import_callback)?;
                }
            }
        }

        Ok(LoadYomitanZipDirResult {
            new_dicts,
            to_be_removed_dicts,
        })
    }

    pub fn import_yomitan_zip_file<Z, Callback>(
        writer: &mut YomitanWriter, // Reuse existing connection
        zip_file: &Z,
        lang: &str,
        progress_callback: Callback,
    ) -> Result<YomitanZipImportResult, CJDicError>
    where
        Z: ZipSource,
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
