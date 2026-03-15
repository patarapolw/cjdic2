use rusqlite::{Result, params};
use serde::Serialize;
use zstd::bulk::Decompressor;

use crate::{
    CJDicError, Timer,
    db::{Database, yomitan_writer::normalize_term},
    tokenizer::Tokenizer,
};

#[derive(Serialize, Debug, Clone)]
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
    tokenizer: Tokenizer,
}

impl YomitanDatabase {
    pub(crate) fn new(db: Database, tokenizer: Tokenizer) -> Self {
        Self { db, tokenizer }
    }

    fn decompressor(&self) -> Result<Decompressor<'_>, CJDicError> {
        let conn = self.db.conn.lock()?;

        let row = conn.query_row(
            "SELECT value FROM glossary.meta WHERE key = 'zstd_dict'",
            [],
            |r| r.get::<_, Vec<u8>>(0),
        )?;

        let decompressor = zstd::bulk::Decompressor::with_dictionary(&row)?;
        Ok(decompressor)
    }

    pub fn search_yomitan(
        &self,
        q_term: &str,
        q_reading: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<YomitanRow>, CJDicError> {
        let q_label = if q_term == q_reading {
            q_term
        } else {
            &format!("{} ({})", q_term, q_reading)
        };

        let _timer = Timer::new(format!(
            "search_yomitan: {} offset={} limit={}",
            q_label, offset, limit
        ));

        // must be run before self.db.conn.lock()
        let mut decompressor = self.decompressor()?;
        // max decompressed: 90KB  (compression ratio on largest: ~2.5x)
        const MAX_DECOMPRESSED_SIZE: usize = 512 * 1024; // 512KB, ~5x headroom

        let is_fts = q_term.starts_with('~');
        let q_term = if is_fts {
            normalize_term(q_term)
        } else {
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
            new_t
        };

        let and_or = if q_term == q_reading { "OR" } else { "AND" };

        let conn = self.db.conn.lock()?;

        conn.execute_batch(
            "
            CREATE TEMP TABLE IF NOT EXISTS search_ids (term_id INTEGER);
            DELETE FROM search_ids;
            ",
        )?;

        if is_fts {
            let q = self.tokenizer.ja_normalize(q_term.to_string()).join(" ");
            println!("fts: {q_term} => {q}");

            if q.len() > 0 {
                conn.execute(
                    &format!(
                        "
                    INSERT INTO search_ids (term_id)
                    SELECT rowid FROM search.terms_ft
                    WHERE term MATCH ?1
                    ORDER BY rank
                    LIMIT ?2 OFFSET ?3
                "
                    ),
                    params![q, limit, offset],
                )?;
            }
        } else {
            let eq1 = if q_term.len() > 0 { "GLOB" } else { "=" };
            let eq2 = if q_reading.len() > 0 { "GLOB" } else { "=" };

            conn.execute(
                &format!(
                    "
                INSERT INTO search_ids (term_id)
                SELECT id FROM search.terms
                WHERE term {eq1} ?1 {and_or} reading {eq2} ?2
                ORDER BY max_score DESC
                LIMIT ?3 OFFSET ?4
            "
                ),
                params![q_term, q_reading, limit, offset],
            )?;
        }

        let mut stmt = conn.prepare(&format!(
            "
            SELECT
                t.term,
                t.reading,
                COALESCE(dt.tags,  '')  AS def_tags,
                COALESCE(r.rules,  '')  AS rules,
                t.score,
                COALESCE(g.b, '[]')     AS glossary,
                t.sequence,
                COALESCE(tt.tags,  '')  AS term_tags,
                d.title                 AS dict_title
            FROM search_ids                 s
            JOIN yomitan.terms              t  ON t.id  = s.term_id
            LEFT JOIN yomitan.dictionaries  d  ON d.id  = t.dict_id
            LEFT JOIN yomitan.def_tag_sets  dt ON dt.id = t.def_tags_id
            LEFT JOIN yomitan.rule_sets     r  ON r.id  = t.rules_id
            LEFT JOIN yomitan.term_tag_sets tt ON tt.id = t.term_tags_id
            LEFT JOIN glossary.glossaries   g  ON g.id  = t.glossary_id
        "
        ))?;
        let rows = stmt.query_map([], |r| {
            Ok((
                YomitanRow {
                    term: r.get(0)?,
                    reading: r.get(1)?,
                    def_tags: r.get(2)?,
                    rules: r.get(3)?,
                    score: r.get(4)?,
                    glossary_json: String::from("[]"), // a valid JSON array
                    sequence: r.get(6)?,
                    term_tags: r.get(7)?,
                    dict_title: r.get(8)?,
                },
                r.get::<_, Vec<u8>>(5)?,
            ))
        })?;

        let mut out = Vec::new();
        for row in rows {
            let (mut r, glossary_b) = row?;
            r.glossary_json =
                String::from_utf8(decompressor.decompress(&glossary_b, MAX_DECOMPRESSED_SIZE)?)?;

            out.push(r);
        }
        Ok(out)
    }
}
