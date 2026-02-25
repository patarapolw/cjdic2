use rusqlite::{Result, params};
use serde::Serialize;

use crate::{
    CJDicError,
    db::{Database, yomitan_writer::normalize_term},
};

#[derive(Serialize, Debug)]
pub struct YomitanRow {
    term: String,
    reading: String,
    def_tags: String,
    rules: String,
    score: i64,
    glossary_json: String,
    sequence: Option<i64>,
    term_tags: String,
    dict_title: String,
}

pub struct YomitanDatabase {
    db: Database,
}

impl YomitanDatabase {
    pub(crate) fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn search_yomitan(
        &self,
        q_term: &str,
        q_reading: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<YomitanRow>, CJDicError> {
        let and_or = if q_term == q_reading { "OR" } else { "AND" };

        let mut new_t = String::new();
        let mut new_seg = String::new();

        for c in q_term.chars() {
            match c {
                '*' | '?' | '[' | ']' | '\\' => {
                    new_t.push_str(&normalize_term(&new_seg));
                    new_seg.clear();
                    new_t.push(c)
                }
                _ => new_seg.push(c),
            }
        }

        new_t.push_str(&normalize_term(&new_seg));

        let q_term_norm = new_t;

        // TODO: consider using = if not GLOB string
        let sql = format!(
            r#"
        SELECT
            t.term,
            t.reading,
            COALESCE(dt.tags,  '')  AS def_tags,
            COALESCE(r.rules,  '')  AS rules,
            t.score,
            g.content               AS glossary_json,
            t.sequence,
            COALESCE(tt.tags,  '')  AS term_tags,
            d.title                 AS dict_title
        FROM yomitan.terms              t
        JOIN yomitan.view_terms_term_rank tr ON tr.term = t.term AND tr.reading = t.reading
        JOIN yomitan.glossaries         g  ON g.id  = t.glossary_id
        JOIN yomitan.dictionaries       d  ON d.id  = t.dict_id
        LEFT JOIN yomitan.def_tag_sets  dt ON dt.id = t.def_tags_id
        LEFT JOIN yomitan.rule_sets     r  ON r.id  = t.rules_id
        LEFT JOIN yomitan.term_tag_sets tt ON tt.id = t.term_tags_id
        WHERE t.term_norm GLOB ?1 {} t.reading GLOB ?2
        ORDER BY tr.max_score DESC, d.sort_order DESC
        LIMIT ?3 OFFSET ?4
        "#,
            and_or
        );

        let conn = self.db.conn.lock().unwrap();

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params![q_term_norm, q_reading, limit, offset], |r| {
            Ok(YomitanRow {
                term: r.get(0)?,
                reading: r.get(1)?,
                def_tags: r.get(2)?,
                rules: r.get(3)?,
                score: r.get(4)?,
                glossary_json: r.get(5)?,
                sequence: r.get(6)?,
                term_tags: r.get(7)?,
                dict_title: r.get(8)?,
            })
        })?;

        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }
}
