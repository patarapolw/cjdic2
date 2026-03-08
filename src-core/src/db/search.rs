use rusqlite::Connection;

use crate::{
    CJDicError,
    db::{DBSCHEMA, DbChild},
};

pub(super) fn normalize_reading(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c >= 'ア' && c <= 'ン' {
                // Katakana range
                if let Some(x) = char::from_u32(c as u32 - 0x60) {
                    x // Convert to Hiragana
                } else {
                    c // Ensure it's a valid char
                }
            } else {
                c // Return the character as is if it's not in the range
            }
        })
        .collect()
}

pub struct SearchDatabase;
//  {
//     db: Database,
//}

// pub struct SearchEntry {
//     id: i64,
//     term: String,
//     reading: String,
// }

impl SearchDatabase {
    // pub(crate) fn new(db: Database) -> Self {
    //     Self { db }
    // }

    pub fn create_schema(conn: &Connection) -> Result<(), CJDicError> {
        conn.execute_batch(
            &format!("
            CREATE TABLE IF NOT EXISTS {0}.terms (
                id          INTEGER PRIMARY KEY,    -- yomitan.terms.id OR yomitan dups OR -user.terms.id
                term        TEXT NOT NULL,          -- normalized
                reading     TEXT NOT NULL           -- normalized
            );

            CREATE INDEX IF NOT EXISTS {0}.terms_term       ON terms (term);
            CREATE INDEX IF NOT EXISTS {0}.terms_reading    ON terms (reading);

            CREATE VIRTUAL TABLE IF NOT EXISTS {0}.terms_ft USING fts5 (
                term,
                content='',
                tokenize=unicode61
            );
        ", DBSCHEMA[DbChild::Search]),
        )?;

        Ok(())
    }

    // pub fn insert_many(
    //     &self,
    //     entries: Vec<SearchEntry>,
    //     tokenizer: Tokenizer,
    // ) -> Result<(), CJDicError> {
    //     let mut conn = self.db.conn.lock()?;
    //     let tx = conn.transaction()?;
    //     {
    //         let mut stmt =
    //             tx.prepare("INSERT INTO terms (id, term, reading) VALUES (?1, ?2, ?3)")?;
    //         for it in entries.iter() {
    //             stmt.execute(params![
    //                 it.id,
    //                 normalize_term(&it.term),
    //                 normalize_reading(&it.reading)
    //             ])?;
    //         }

    //         let exclude_pos =
    //             HashSet::from(["助詞", "助動詞", "記号", "接続詞"].map(|p| p.to_string()));

    //         let mut stmt = tx.prepare("INSERT INTO terms_ft (rowid, term) VALUES (?1, ?2)")?;
    //         for it in entries {
    //             let mut segmented: Vec<String> = vec![];
    //             for t in tokenizer.tokenize(it.term) {
    //                 let mut base = t.surface;
    //                 let mut pos = None;

    //                 for (i, dt) in t.details.iter().enumerate() {
    //                     match i {
    //                         0 => base = dt.to_string(),
    //                         6 => pos = Some(dt),
    //                         _ => {}
    //                     }
    //                 }

    //                 if let Some(p) = pos {
    //                     if exclude_pos.contains(p.as_str()) {
    //                         continue;
    //                     }
    //                 }
    //                 segmented.push(base);
    //             }

    //             stmt.execute(params![it.id, segmented.join(" ")])?;
    //         }
    //     }
    //     tx.commit()?;

    //     Ok(())
    // }
}
