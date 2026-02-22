use std::{collections::HashMap, fs::File, io::Read, path::PathBuf};

use anyhow::Context;
use blake3::hash;
use rusqlite::{Connection, Transaction, params};
use serde::Serialize;
use serde_json::Value;
use zip::ZipArchive;

use crate::CJDicError;

fn blake3_hex(s: &str) -> String {
    format!("{}", hash(s.as_bytes()))
}

pub fn create_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "
        PRAGMA journal_mode = WAL;
        PRAGMA foreign_keys = ON;
        PRAGMA synchronous  = NORMAL;
        PRAGMA cache_size   = -65536;
        PRAGMA temp_store   = MEMORY;

        CREATE TABLE IF NOT EXISTS schema_meta (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS dictionaries (
            id           INTEGER PRIMARY KEY,
            title        TEXT    NOT NULL,
            revision     TEXT    NOT NULL,
            author       TEXT,
            url          TEXT,
            description  TEXT,
            sort_order   INTEGER NOT NULL DEFAULT 0,
            installed_at TEXT    NOT NULL DEFAULT (datetime('now')),
            lang         TEXT    NOT NULL,
            UNIQUE (title, revision)
        );
        CREATE TABLE IF NOT EXISTS glossaries (
            id      INTEGER PRIMARY KEY,
            hash    TEXT NOT NULL UNIQUE,
            content TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS def_tag_sets (id INTEGER PRIMARY KEY, tags TEXT NOT NULL UNIQUE);
        CREATE TABLE IF NOT EXISTS term_tag_sets (id INTEGER PRIMARY KEY, tags TEXT NOT NULL UNIQUE);
        CREATE TABLE IF NOT EXISTS rule_sets (id INTEGER PRIMARY KEY, rules TEXT NOT NULL UNIQUE);
        CREATE TABLE IF NOT EXISTS terms (
            id INTEGER PRIMARY KEY,
            dict_id INTEGER NOT NULL REFERENCES dictionaries(id) ON DELETE CASCADE,
            term TEXT NOT NULL,
            reading TEXT NOT NULL,
            def_tags_id INTEGER,
            rules_id INTEGER,
            score INTEGER NOT NULL DEFAULT 0,
            glossary_id INTEGER NOT NULL REFERENCES glossaries(id),
            sequence INTEGER,
            term_tags_id INTEGER
        );
        CREATE TABLE IF NOT EXISTS term_meta (
            id INTEGER PRIMARY KEY,
            dict_id INTEGER NOT NULL REFERENCES dictionaries(id) ON DELETE CASCADE,
            term TEXT NOT NULL,
            mode TEXT NOT NULL,
            reading TEXT,
            data TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS tags (
            id INTEGER PRIMARY KEY,
            dict_id INTEGER NOT NULL REFERENCES dictionaries(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            category TEXT,
            sort_order INTEGER NOT NULL DEFAULT 0,
            notes TEXT,
            score INTEGER NOT NULL DEFAULT 0,
            UNIQUE (dict_id, name)
        );
        CREATE TABLE IF NOT EXISTS kanji (
            id INTEGER PRIMARY KEY,
            dict_id INTEGER NOT NULL REFERENCES dictionaries(id) ON DELETE CASCADE,
            kanji TEXT NOT NULL,
            onyomi TEXT,
            kunyomi TEXT,
            tags TEXT,
            meanings TEXT NOT NULL DEFAULT '[]',
            stats TEXT NOT NULL DEFAULT '{}'
        );
        ",
    )?;

    conn.execute_batch(
        "
        CREATE INDEX IF NOT EXISTS dictionaries_lang ON dictionaries (lang);

        CREATE INDEX IF NOT EXISTS terms_term ON terms (term);
        CREATE INDEX IF NOT EXISTS terms_reading ON terms (reading);
        ",
    )?;

    let mut stmt = conn.prepare("SELECT 1 FROM schema_meta WHERE key = 'schema_version'")?;
    if stmt.exists([])? == false {
        conn.prepare("INSERT INTO schema_meta (key, value) VALUES ('schema_version', '1')")?
            .execute([])?;
        conn.prepare(
            "INSERT INTO schema_meta (key, value) VALUES ('created_at', datetime('now'))",
        )?
        .execute([])?;
    }

    Ok(())
}

fn intern(
    tx: &Transaction,
    insert_sql: &str,
    select_sql: &str,
    cache: &mut HashMap<String, i64>,
    value: &str,
) -> rusqlite::Result<i64> {
    if let Some(&v) = cache.get(value) {
        return Ok(v);
    }
    tx.execute(insert_sql, [value])?;
    let id: i64 = tx.query_row(select_sql, [value], |r| r.get(0))?;
    cache.insert(value.to_string(), id);
    Ok(id)
}

#[derive(Serialize, Debug)]
pub struct YomitanZipImportResult {
    exists: bool,
    load: bool,
    error: Option<String>,
}

pub fn import_bundled_zip_file(
    conn: &mut Connection,
    zip_path: PathBuf,
    lang: &str,
) -> anyhow::Result<YomitanZipImportResult, CJDicError> {
    create_schema(&conn)?;

    let f = File::open(&zip_path).with_context(|| format!("opening zip {}", zip_path.display()))?;
    let mut archive = ZipArchive::new(f).with_context(|| "reading zip archive")?;

    let index_file = match archive.by_name("index.json") {
        Ok(mut f) => {
            let mut s = String::new();
            f.read_to_string(&mut s)?;
            serde_json::from_str::<Value>(&s)?
        }
        Err(e) => {
            // TODO: proper error handling
            eprintln!("{}", e);
            return Ok(YomitanZipImportResult {
                exists: false,
                load: false,
                error: Some(format!("{}", e)),
            });
        }
    };

    let title = index_file
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let revision = index_file
        .get("revision")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();

    // Skip if already installed
    let exists: bool = conn
        .prepare("SELECT 1 FROM dictionaries WHERE title = ?1 AND revision = ?2")?
        .exists([title, revision])?;
    if exists {
        return Ok(YomitanZipImportResult {
            exists: true,
            load: false,
            error: None,
        });
    }

    let tx = conn.transaction()?;
    tx.execute(
            "INSERT INTO dictionaries (title, revision, author, url, description, lang) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                index_file.get("title").and_then(Value::as_str),
                index_file.get("revision").and_then(Value::as_str),
                index_file.get("author").and_then(Value::as_str),
                index_file.get("url").and_then(Value::as_str),
                index_file.get("description").and_then(Value::as_str),
                lang,
            ],
        )?;

    let dict_id: i64 = tx.query_row("SELECT last_insert_rowid()", [], |r| r.get(0))?;

    // prepare statements
    let insert_glossary = "INSERT OR IGNORE INTO glossaries (hash, content) VALUES (?1, ?2)";
    let select_glossary = "SELECT id FROM glossaries WHERE hash = ?1";
    let insert_def = "INSERT OR IGNORE INTO def_tag_sets (tags) VALUES (?1)";
    let select_def = "SELECT id FROM def_tag_sets WHERE tags = ?1";
    let insert_term_tags = "INSERT OR IGNORE INTO term_tag_sets (tags) VALUES (?1)";
    let select_term_tags = "SELECT id FROM term_tag_sets WHERE tags = ?1";
    let insert_rules = "INSERT OR IGNORE INTO rule_sets (rules) VALUES (?1)";
    let select_rules = "SELECT id FROM rule_sets WHERE rules = ?1";
    let insert_term = "INSERT INTO terms (dict_id, term, reading, def_tags_id, rules_id, score, glossary_id, sequence, term_tags_id) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)";
    let insert_meta =
        "INSERT INTO term_meta (dict_id, term, mode, reading, data) VALUES (?1,?2,?3,?4,?5)";
    let insert_tag = "INSERT OR IGNORE INTO tags (dict_id, name, category, sort_order, notes, score) VALUES (?1,?2,?3,?4,?5,?6)";

    let mut glossary_cache: HashMap<String, i64> = HashMap::new();
    let mut def_cache: HashMap<String, i64> = HashMap::new();
    let mut term_tags_cache: HashMap<String, i64> = HashMap::new();
    let mut rules_cache: HashMap<String, i64> = HashMap::new();

    // term banks
    let mut bank_i = 1;
    loop {
        let name = format!("term_bank_{}.json", bank_i);
        match archive.by_name(&name) {
            Ok(mut f) => {
                let mut s = String::new();
                f.read_to_string(&mut s)?;
                let entries: Vec<Value> = serde_json::from_str(&s)?;
                for e in entries {
                    let term = e.get(0).and_then(Value::as_str).unwrap_or("");
                    let reading = e.get(1).and_then(Value::as_str).unwrap_or("");
                    let def_tags = e
                        .get(2)
                        .and_then(Value::as_str)
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string());
                    let rules = e
                        .get(3)
                        .and_then(Value::as_str)
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string());
                    let score = e.get(4).and_then(Value::as_i64).unwrap_or(0);
                    let glossary_val = e.get(5).cloned().unwrap_or(Value::Null);
                    let sequence = e.get(6).and_then(Value::as_i64);
                    let term_tags = e
                        .get(7)
                        .and_then(Value::as_str)
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string());

                    let glossary_json = serde_json::to_string(&glossary_val)?;
                    let hash = blake3_hex(&glossary_json);
                    let glossary_id = if let Some(&id) = glossary_cache.get(&hash) {
                        id
                    } else {
                        tx.execute(insert_glossary, params![hash, glossary_json])?;
                        let id: i64 = tx.query_row(select_glossary, params![hash], |r| r.get(0))?;
                        glossary_cache.insert(hash.clone(), id);
                        id
                    };

                    let def_id = if let Some(s) = def_tags.as_deref() {
                        intern(&tx, insert_def, select_def, &mut def_cache, s)?
                    } else {
                        0
                    };
                    let rules_id = if let Some(s) = rules.as_deref() {
                        intern(&tx, insert_rules, select_rules, &mut rules_cache, s)?
                    } else {
                        0
                    };
                    let term_tags_id = if let Some(s) = term_tags.as_deref() {
                        intern(
                            &tx,
                            insert_term_tags,
                            select_term_tags,
                            &mut term_tags_cache,
                            s,
                        )?
                    } else {
                        0
                    };

                    tx.execute(
                        insert_term,
                        params![
                            dict_id,
                            term,
                            reading,
                            if def_id != 0 {
                                Some(def_id)
                            } else {
                                Option::<i64>::None
                            },
                            if rules_id != 0 {
                                Some(rules_id)
                            } else {
                                Option::<i64>::None
                            },
                            score,
                            glossary_id,
                            sequence,
                            if term_tags_id != 0 {
                                Some(term_tags_id)
                            } else {
                                Option::<i64>::None
                            },
                        ],
                    )?;
                }
                bank_i += 1;
                continue;
            }
            Err(_) => break,
        }
    }

    // term_meta banks
    let mut meta_i = 1;
    loop {
        let name = format!("term_meta_bank_{}.json", meta_i);
        match archive.by_name(&name) {
            Ok(mut f) => {
                let mut s = String::new();
                f.read_to_string(&mut s)?;
                let entries: Vec<Value> = serde_json::from_str(&s)?;
                for e in entries {
                    let term = e.get(0).and_then(Value::as_str).unwrap_or("");
                    let mode = e.get(1).and_then(Value::as_str).unwrap_or("");
                    let data = e.get(2).cloned().unwrap_or(Value::Null);
                    let reading = data
                        .get("reading")
                        .and_then(Value::as_str)
                        .map(|s| s.to_string());
                    tx.execute(
                        insert_meta,
                        params![dict_id, term, mode, reading, serde_json::to_string(&data)?],
                    )?;
                }
                meta_i += 1;
                continue;
            }
            Err(_) => break,
        }
    }

    // tag banks
    let mut tag_i = 1;
    loop {
        let name = format!("tag_bank_{}.json", tag_i);
        match archive.by_name(&name) {
            Ok(mut f) => {
                let mut s = String::new();
                f.read_to_string(&mut s)?;
                let entries: Vec<Value> = serde_json::from_str(&s)?;
                for e in entries {
                    let name = e.get(0).and_then(Value::as_str).unwrap_or("");
                    let category = e.get(1).and_then(Value::as_str);
                    let sort_order = e.get(2).and_then(Value::as_i64).unwrap_or(0);
                    let notes = e.get(3).and_then(Value::as_str);
                    let tag_score = e.get(4).and_then(Value::as_i64).unwrap_or(0);
                    tx.execute(
                        insert_tag,
                        params![dict_id, name, category, sort_order, notes, tag_score],
                    )?;
                }
                tag_i += 1;
                continue;
            }
            Err(_) => break,
        }
    }

    tx.commit()?;

    conn.execute_batch("VACUUM;")?;

    Ok(YomitanZipImportResult {
        exists: true,
        load: true,
        error: None,
    })
}

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

pub fn search_yomitan(
    conn: &Connection,
    q_term: &str,
    q_reading: &str,
    limit: u32,
    offset: u32,
) -> Result<Vec<YomitanRow>, CJDicError> {
    let and_or = if q_term == q_reading { "OR" } else { "AND" };

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
        FROM terms t
        JOIN  glossaries    g  ON g.id  = t.glossary_id
        JOIN  dictionaries  d  ON d.id  = t.dict_id
        LEFT JOIN def_tag_sets  dt ON dt.id = t.def_tags_id
        LEFT JOIN rule_sets      r ON r.id  = t.rules_id
        LEFT JOIN term_tag_sets tt ON tt.id = t.term_tags_id
        WHERE t.term GLOB ?1 {} t.reading GLOB ?2
        ORDER BY t.score DESC
        LIMIT ?3 OFFSET ?4
        "#,
        and_or
    );

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![q_term, q_reading, limit, offset], |r| {
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
