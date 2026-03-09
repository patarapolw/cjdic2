use std::collections::HashSet;

use rusqlite::{Connection, params};
use wana_kana::utils::katakana_to_hiragana;

use crate::{
    CJDicError, Timer, YomitanProgress,
    db::{DBFILE, DBSCHEMA, DbChild, yomitan_writer::normalize_term},
    tokenizer::Tokenizer,
};

pub(super) fn normalize_reading(s: &str) -> String {
    katakana_to_hiragana(s)
}

pub struct SearchDatabase<'a> {
    conn: &'a mut Connection,
}

impl<'a> SearchDatabase<'a> {
    pub(crate) fn new(conn: &'a mut Connection) -> Self {
        Self { conn }
    }

    pub fn create_schema(&self) -> Result<(), CJDicError> {
        self.conn.execute_batch(
            &format!("
            CREATE TABLE IF NOT EXISTS {0}.terms (
                id          INTEGER PRIMARY KEY,    -- yomitan.terms.id OR yomitan dups OR -user.terms.id
                term        TEXT NOT NULL,          -- normalized
                reading     TEXT NOT NULL,          -- normalized
                max_score   INTEGER
            );

            CREATE INDEX IF NOT EXISTS {0}.terms_term       ON terms (term);
            CREATE INDEX IF NOT EXISTS {0}.terms_reading    ON terms (reading);
            CREATE INDEX IF NOT EXISTS {0}.terms_max_score  ON terms (max_score DESC);

            CREATE VIRTUAL TABLE IF NOT EXISTS {0}.terms_ft USING fts5 (
                term,
                content='',
                tokenize=unicode61
            );
        ", DBSCHEMA[DbChild::Search]),
        )?;

        Ok(())
    }

    pub fn reset_db(&self) -> Result<(), CJDicError> {
        self.conn.execute_batch(&format!(
            "
            DELETE FROM {0}.terms;
            DELETE FROM {0}.terms_ft;
        ",
            DBSCHEMA[DbChild::Search]
        ))?;

        Ok(())
    }

    pub fn regenerate_yomitan(
        &mut self,
        yomitan_schema: &str,
        tokenizer: Tokenizer,
        progress_callback: impl Fn(YomitanProgress),
    ) -> Result<(), CJDicError> {
        let total = self.conn.query_row(
            &format!("SELECT COALESCE(MAX(id), 0) FROM {}.terms", yomitan_schema),
            [],
            |r| r.get::<_, i64>(0),
        )? as usize;

        if total == 0 {
            return Ok(());
        }

        let message = &format!("Generating {}", DBFILE[DbChild::Search]);
        progress_callback(YomitanProgress {
            message: message.to_string(),
            current: 0,
            total,
            steps: 0,
        });

        let _timer = Timer::new(message.to_string());

        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare(&format!(
                "SELECT t.id, t.term, t.reading, tr.max_score
                FROM {0}.terms t
                LEFT JOIN {0}.view_terms_term_rank tr ON tr.term = t.term AND tr.reading = t.reading
                ",
                yomitan_schema
            ))?;

            let rows = stmt.query_map([], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, Option<i64>>(3)?,
                ))
            })?;

            let mut stmt_write = tx.prepare(&format!(
                "INSERT INTO {}.terms (id, term, reading, max_score) VALUES (?1, ?2, ?3, ?4)",
                DBSCHEMA[DbChild::Search]
            ))?;

            let exclude_pos =
                HashSet::from(["助詞", "助動詞", "記号", "接続詞"].map(|p| p.to_string()));
            let mut stmt_write_ft = tx.prepare(&format!(
                "INSERT INTO {}.terms_ft (rowid, term) VALUES (?1, ?2)",
                DBSCHEMA[DbChild::Search]
            ))?;

            for (i, row) in rows.enumerate() {
                let (id, term, reading, max_score) = row?;
                stmt_write.execute(params![
                    id,
                    normalize_term(&term),
                    normalize_reading(&reading),
                    max_score
                ])?;

                let mut segmented: Vec<String> = vec![];
                for t in tokenizer.tokenize(term) {
                    let mut base = t.surface;
                    let mut pos = None;

                    for (i, dt) in t.details.iter().enumerate() {
                        match i {
                            0 => base = dt.to_string(),
                            6 => pos = Some(dt),
                            _ => {}
                        }
                    }

                    if let Some(p) = pos {
                        if exclude_pos.contains(p.as_str()) {
                            continue;
                        }
                    }
                    segmented.push(base);
                }

                stmt_write_ft.execute(params![id, segmented.join(" ")])?;

                if i % 10_000 == 0 {
                    progress_callback(YomitanProgress {
                        message: message.to_string(),
                        current: i,
                        total,
                        steps: i,
                    });
                }
            }
        }
        tx.commit()?;

        Ok(())
    }
}
