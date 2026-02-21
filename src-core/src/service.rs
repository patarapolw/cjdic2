use std::path::Path;

use crate::{db::Database, error::CJDicError, models::Entry};

pub struct AppService {
    db: Database,
}

impl AppService {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self, CJDicError> {
        let db = Database::new(db_path)?;
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
}
