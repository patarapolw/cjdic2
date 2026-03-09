use blake3::hash;
use rusqlite::{Connection, OptionalExtension, Result, Transaction, params};
use serde::Serialize;
use serde_json::Value;
use std::{
    borrow::Cow,
    collections::HashMap,
    env::current_dir,
    io::{Cursor, Read},
    path::{Path, PathBuf},
};
use zip::ZipArchive;

use crate::{
    CJDicError, Timer,
    db::{DBFILE, DBSCHEMA, DbChild, search::SearchDatabase},
    tokenizer::Tokenizer,
};

fn blake3_hex(s: &str) -> String {
    format!("{}", hash(s.as_bytes()))
}

pub(super) fn normalize_term(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphabetic())
        .flat_map(|c| c.to_lowercase())
        .collect()
}

pub trait ZipSource {
    fn file_name(&self) -> &str;
    fn bytes(&self) -> std::io::Result<Cow<'_, [u8]>>;
}

impl ZipSource for PathBuf {
    fn file_name(&self) -> &str {
        Path::file_name(self).and_then(|n| n.to_str()).unwrap_or("")
    }

    fn bytes(&self) -> std::io::Result<Cow<'_, [u8]>> {
        std::fs::read(self).map(Cow::Owned)
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct YomitanProgress {
    pub message: String,
    pub current: usize,
    pub total: usize,
    pub steps: usize,
}

#[derive(Serialize, Debug)]
pub struct YomitanZipImportResult {
    pub exists: bool,
    pub load: bool,
    pub error: Option<String>,
}

pub struct YomitanWriter {
    dir: PathBuf,
    conn: Connection,
}

impl YomitanWriter {
    pub fn new(db_dir: impl AsRef<Path>) -> Result<Self> {
        let dir = db_dir.as_ref().to_path_buf();
        let conn = Connection::open(dir.join(DBFILE[DbChild::Yomitan]))?;

        conn.execute_batch(
            "
            PRAGMA journal_mode = WAL;
            PRAGMA foreign_keys = ON;
            PRAGMA synchronous  = NORMAL;
            PRAGMA cache_size   = -65536;
            PRAGMA temp_store   = MEMORY;
            ",
        )?;

        Ok(Self { dir, conn })
    }

    pub fn create_schema(
        &mut self,
        progress_callback: impl Fn(YomitanProgress),
    ) -> Result<(), CJDicError> {
        self.conn.execute_batch(
        "
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
                bundle_name  TEXT,                  -- custom
                lang         TEXT    NOT NULL,      -- custom
                UNIQUE (title, revision)
            );
            CREATE TABLE IF NOT EXISTS glossaries (     -- not used. moved to a new file
                id              INTEGER PRIMARY KEY,
                hash            TEXT NOT NULL UNIQUE,           -- maybe used in the future for excerpts
                content         TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS def_tag_sets (id INTEGER PRIMARY KEY, tags TEXT NOT NULL UNIQUE);
            CREATE TABLE IF NOT EXISTS term_tag_sets (id INTEGER PRIMARY KEY, tags TEXT NOT NULL UNIQUE);
            CREATE TABLE IF NOT EXISTS rule_sets (id INTEGER PRIMARY KEY, rules TEXT NOT NULL UNIQUE);
            CREATE TABLE IF NOT EXISTS terms (
                id              INTEGER PRIMARY KEY,
                dict_id         INTEGER NOT NULL REFERENCES dictionaries(id) ON DELETE CASCADE,
                term            TEXT NOT NULL,
                reading         TEXT NOT NULL,
                def_tags_id     INTEGER,
                rules_id        INTEGER,
                score           INTEGER NOT NULL DEFAULT 0,
                glossary_id     INTEGER NOT NULL REFERENCES glossaries(id),
                sequence        INTEGER,
                term_tags_id    INTEGER
            );
            CREATE TABLE IF NOT EXISTS term_meta (
                id              INTEGER PRIMARY KEY,
                dict_id         INTEGER NOT NULL REFERENCES dictionaries(id) ON DELETE CASCADE,
                term            TEXT NOT NULL,
                mode            TEXT NOT NULL,
                reading         TEXT,
                data            TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS tags (
                id              INTEGER PRIMARY KEY,
                dict_id         INTEGER NOT NULL REFERENCES dictionaries(id) ON DELETE CASCADE,
                name            TEXT NOT NULL,
                category        TEXT,
                sort_order      INTEGER NOT NULL DEFAULT 0,
                notes           TEXT,
                score           INTEGER NOT NULL DEFAULT 0,
                UNIQUE (dict_id, name)
            );
            CREATE TABLE IF NOT EXISTS kanji (
                id              INTEGER PRIMARY KEY,
                dict_id         INTEGER NOT NULL REFERENCES dictionaries(id) ON DELETE CASCADE,
                kanji           TEXT NOT NULL,
                onyomi          TEXT,
                kunyomi         TEXT,
                tags            TEXT,
                meanings        TEXT NOT NULL DEFAULT '[]',
                stats           TEXT NOT NULL DEFAULT '{}'
            );
            ",
        )?;

        self.conn.execute_batch(
            "
            CREATE INDEX IF NOT EXISTS dictionaries_sort_order ON dictionaries (sort_order DESC);
            CREATE INDEX IF NOT EXISTS dictionaries_lang ON dictionaries (lang);

            CREATE INDEX IF NOT EXISTS terms_dict_id ON terms (dict_id);
            CREATE INDEX IF NOT EXISTS terms_term_reading_score ON terms(term, reading, score DESC);

            CREATE INDEX IF NOT EXISTS term_meta_dict_id ON term_meta (dict_id);

            CREATE INDEX IF NOT EXISTS tags_dict_id ON tags (dict_id);

            CREATE INDEX IF NOT EXISTS kanji_dict_id ON kanji (dict_id);
            ",
        )?;

        self.create_materialized_views()?;
        self.attach_db(DbChild::YomitanGlossary)?;
        self.attach_db(DbChild::Search)?;

        let schema_version = "2026-03-08";
        let current_schema_version = {
            let mut stmt = self
                .conn
                .prepare("SELECT value FROM schema_meta WHERE key = 'schema_version'")?;

            stmt.query_one([], |r| r.get::<_, String>(0)).optional()?
        };
        if let Some(v) = current_schema_version {
            let v = v.as_str();
            if v < "2" {
                let total =
                    self.conn
                        .query_row("SELECT COALESCE(MAX(id), 0) FROM glossaries", [], |r| {
                            r.get::<_, i64>(0)
                        })? as usize;

                let message = &format!("Migrate to {}", DBFILE[DbChild::YomitanGlossary]);
                progress_callback(YomitanProgress {
                    message: message.to_string(),
                    current: 0,
                    total,
                    steps: 0,
                });

                let _timer = Timer::new(message.to_string());

                let tx = self.conn.transaction()?;
                {
                    let mut stmt = tx.prepare("SELECT id, hash, content FROM glossaries")?;

                    let rows = stmt.query_map([], |r| {
                        Ok((
                            r.get::<_, i64>(0)?,
                            r.get::<_, String>(1)?,
                            r.get::<_, String>(2)?,
                        ))
                    })?;

                    let mut stmt_write = tx.prepare(
                        "INSERT INTO glossary.glossaries (id, hash, b) VALUES (?1, ?2, ?3)",
                    )?;

                    for (i, row) in rows.enumerate() {
                        let (id, hash, glossary_json) = row?;
                        stmt_write.execute(params![
                            id,
                            hash,
                            zstd::encode_all(glossary_json.as_bytes(), 3)?
                        ])?;

                        if i % 1000 == 0 {
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

                progress_callback(YomitanProgress {
                    message: message.to_string(),
                    current: total,
                    total,
                    steps: total,
                });

                self.hash_glossary()?;

                progress_callback(YomitanProgress {
                    message: "Cleaning up old glossary".to_string(),
                    current: 0,
                    total: 0,
                    steps: total,
                });

                let _timer =
                    Timer::new(format!("cleanup for {}", DBFILE[DbChild::YomitanGlossary]));
                self.conn.execute_batch(
                    "
                    UPDATE glossaries SET content = '[]';
                    VACUUM;
                ",
                )?;
            }

            if v < schema_version {
                self.conn
                    .prepare("UPDATE schema_meta SET value = ?2 WHERE key = ?1")?
                    .execute(["schema_version", schema_version])?;
            }
        } else {
            self.conn
                .prepare("INSERT INTO schema_meta (key, value) VALUES (?1, ?2)")?
                .execute(["schema_version", schema_version])?;
            self.conn
                .prepare(
                    "INSERT INTO schema_meta (key, value) VALUES ('created_at', datetime('now'))",
                )?
                .execute([])?;
        }

        Ok(())
    }

    fn create_materialized_views(&mut self) -> Result<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS view_terms_term_rank (
                term        TEXT NOT NULL,
                reading     TEXT NOT NULL,
                max_score   INTEGER NOT NULL,
                PRIMARY KEY (term, reading)
            );
            ",
        )?;

        self.conn.execute_batch(
            "
            CREATE TRIGGER IF NOT EXISTS terms_max_score_after_insert
            AFTER INSERT ON terms
            WHEN NEW.score > 0
            -- This automatically excludes NULL (since NULL > 0 is false)
            BEGIN
                INSERT INTO view_terms_term_rank (term, reading, max_score)
                VALUES (NEW.term, NEW.reading, NEW.score)
                ON CONFLICT(term, reading)
                DO UPDATE SET max_score =
                    CASE
                        WHEN NEW.score > max_score
                        THEN NEW.score
                        ELSE max_score
                    END;
            END;
            ",
        )?;
        // no AFTER DELETE trigger yet. existence of max score is more important.

        {
            let mut stmt = self.conn.prepare("SELECT 1 FROM view_terms_term_rank")?;
            if !(stmt.exists([])?) {
                self.conn.execute_batch(
                    "
                    INSERT INTO view_terms_term_rank
                    SELECT term, reading, MAX(score)
                    FROM terms
                    GROUP BY term, reading
                    ",
                )?;
                // 1.9 sec
            }
        }

        Ok(())
    }

    fn attach_db(&mut self, db_child: DbChild) -> Result<(), CJDicError> {
        let dir = self.dir.clone();
        let path = if self.dir.is_absolute() {
            dir
        } else {
            current_dir()?.join(dir)
        };

        let db_file = DBFILE[db_child];
        let schema_name = DBSCHEMA[db_child];

        let path = path.join(db_file).to_string_lossy().replace(r"\", r"/");

        #[cfg(windows)]
        let path = if !path.starts_with("/") {
            format!("/{}", path)
        } else {
            path.to_string()
        };

        let uri = format!("file:{}", path);

        self.conn
            .execute(&format!("ATTACH DATABASE ?1 AS {}", schema_name), [uri])?;
        self.conn.execute_batch(&format!(
            r"
                PRAGMA {}.mmap_size = 3000000000; -- 3_000_000_000
                ",
            schema_name
        ))?;

        match db_child {
            DbChild::YomitanGlossary => self.create_schema_glossary()?,
            DbChild::Search => SearchDatabase::new(&mut self.conn).create_schema()?,
            _ => (),
        }

        Ok(())
    }

    fn create_schema_glossary(&self) -> Result<()> {
        self.conn.execute_batch(&format!(
            "
            CREATE TABLE IF NOT EXISTS glossary.meta (
                key     TEXT PRIMARY KEY,
                value   BLOB NOT NULL
            );

            CREATE TABLE IF NOT EXISTS glossary.glossaries (
                id      INTEGER PRIMARY KEY,
                hash    TEXT NOT NULL UNIQUE,
                b       BLOB NOT NULL
            );
        "
        ))?;

        Ok(())
    }

    fn hash_glossary(&self) -> Result<(), CJDicError> {
        let max_id: i64 =
            self.conn
                .query_row("SELECT MAX(id) FROM glossary.glossaries", [], |r| r.get(0))?;

        let step = max_id / 1000;

        let samples: Vec<Vec<u8>> = self
            .conn
            .prepare("SELECT b FROM glossary.glossaries WHERE id % ?1 = 0 LIMIT 1000")?
            .query_map(params![step], |r| r.get::<_, Vec<u8>>(0))?
            .filter_map(|r| r.ok())
            .collect();

        let dict_size = 112640; // 110KB default
        let dictionary = zstd::dict::from_samples(&samples, dict_size)?;

        self.conn.execute(
            "INSERT OR REPLACE INTO glossary.meta (key, value) VALUES (?1, ?2)",
            params!["zstd_dict", dictionary],
        )?;

        Ok(())
    }

    pub fn import_dictionary_zip_file<Z, Callback>(
        &mut self,
        zip_file: &Z,
        lang: &str,
        progress_callback: Callback,
    ) -> anyhow::Result<YomitanZipImportResult, CJDicError>
    where
        Z: ZipSource,
        Callback: Fn(YomitanProgress),
    {
        let bundle_name = zip_file.file_name();
        let _timer = Timer::new(format!("Importing {}", bundle_name));

        self.conn.execute_batch("PRAGMA foreign_keys = off")?;

        let cursor = Cursor::new(zip_file.bytes()?);
        let mut archive = ZipArchive::new(cursor)?;

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

        let mut steps: usize = 0;
        progress_callback(YomitanProgress {
            message: format!("Opening {}", bundle_name),
            current: 0,
            total: 0,
            steps,
        });

        fn remove_timestamp(s: &str) -> String {
            if let Some(pos) = s.rfind('[') {
                // Trim whitespace only before the '[' and return the substring
                let trimmed_before_bracket = s[..pos].trim_end();
                trimmed_before_bracket.to_string()
            } else {
                s.to_string() // Return the original string if no '[' is found
            }
        }

        let title = remove_timestamp(
            &index_file
                .get("title")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
        );

        let revision = index_file
            .get("revision")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();

        {
            let title: &str = &title;
            // Skip if already installed
            let mut stmt = self
                .conn
                .prepare("SELECT (revision < ?2) FROM dictionaries WHERE title = ?1")?;
            let mut rows = stmt.query(params![title, revision])?;
            if let Some(row) = rows.next()? {
                let is_outdated: bool = row.get(0)?;
                if is_outdated {
                    self.drop_dictionary(bundle_name, lang)?;
                } else {
                    return Ok(YomitanZipImportResult {
                        exists: true,
                        load: false,
                        error: None,
                    });
                }
            }
        }

        self.attach_db(DbChild::YomitanGlossary)?;

        {
            let tx = self.conn.transaction()?;
            tx.execute(
                "INSERT INTO dictionaries
                    (title, revision, author, url, description, lang, bundle_name) VALUES
                    ( ?1,      ?2,      ?3,   ?4,      ?5,       ?6,     ?7)",
                params![
                    title,
                    revision,
                    index_file.get("author").and_then(Value::as_str),
                    index_file.get("url").and_then(Value::as_str),
                    index_file.get("description").and_then(Value::as_str),
                    lang,
                    bundle_name,
                ],
            )?;

            let dict_id: i64 = tx.query_row("SELECT last_insert_rowid()", [], |r| r.get(0))?;

            // prepare statements
            let insert_glossary =
                "INSERT OR IGNORE INTO glossary.glossaries (hash, b) VALUES (?1, ?2)";
            let select_glossary = "SELECT id FROM glossary.glossaries WHERE hash = ?1";
            let insert_def = "INSERT OR IGNORE INTO def_tag_sets (tags) VALUES (?1)";
            let select_def = "SELECT id FROM def_tag_sets WHERE tags = ?1";
            let insert_term_tags = "INSERT OR IGNORE INTO term_tag_sets (tags) VALUES (?1)";
            let select_term_tags = "SELECT id FROM term_tag_sets WHERE tags = ?1";
            let insert_rules = "INSERT OR IGNORE INTO rule_sets (rules) VALUES (?1)";
            let select_rules = "SELECT id FROM rule_sets WHERE rules = ?1";
            let insert_term = "INSERT INTO terms (dict_id, term, reading, def_tags_id, rules_id, score, glossary_id, sequence, term_tags_id) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)";
            let insert_meta = "INSERT INTO term_meta (dict_id, term, mode, reading, data) VALUES (?1,?2,?3,?4,?5)";
            let insert_tag = "INSERT OR IGNORE INTO tags (dict_id, name, category, sort_order, notes, score) VALUES (?1,?2,?3,?4,?5,?6)";

            let mut glossary_cache: HashMap<String, i64> = HashMap::new();
            let mut def_cache: HashMap<String, i64> = HashMap::new();
            let mut term_tags_cache: HashMap<String, i64> = HashMap::new();
            let mut rules_cache: HashMap<String, i64> = HashMap::new();

            let mut n_bank: usize = 0;

            for filename in archive.file_names() {
                let starts_with = "term_bank_";

                if filename.starts_with(starts_with) {
                    let text = filename.to_string();
                    let start_idx = starts_with.len();

                    if let Some(stop_idx) = text[start_idx..].find(".") {
                        let sub = &text[start_idx..(start_idx + stop_idx)];
                        if let Some(i_bank) = sub.parse::<usize>().ok() {
                            if i_bank > n_bank {
                                n_bank = i_bank;
                            }
                        }
                    }
                }
            }

            // term banks
            let mut bank_i = 1;
            loop {
                let name = format!("term_bank_{}.json", bank_i);
                match archive.by_name(&name) {
                    Ok(mut f) => {
                        let mut s = String::new();
                        f.read_to_string(&mut s)?;
                        let entries: Vec<Value> = serde_json::from_str(&s)?;

                        steps += entries.len();
                        progress_callback(YomitanProgress {
                            message: format!("Reading term banks"),
                            current: bank_i,
                            total: n_bank,
                            steps,
                        });

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
                                tx.execute(
                                    insert_glossary,
                                    params![hash, zstd::encode_all(glossary_json.as_bytes(), 3)?],
                                )?;
                                let id: i64 =
                                    tx.query_row(select_glossary, params![hash], |r| r.get(0))?;
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

            steps += 1;
            progress_callback(YomitanProgress {
                message: format!("Reading term meta banks"),
                current: 0,
                total: 0,
                steps,
            });

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
                                params![
                                    dict_id,
                                    term,
                                    mode,
                                    reading,
                                    serde_json::to_string(&data)?
                                ],
                            )?;
                        }
                        meta_i += 1;
                        continue;
                    }
                    Err(_) => break,
                }
            }

            steps += 1;
            progress_callback(YomitanProgress {
                message: format!("Reading tag banks"),
                current: 0,
                total: 0,
                steps,
            });

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
        }

        self.conn.execute_batch(
            "
            PRAGMA foreign_keys = on;
            PRAGMA check_foreign_keys;
            ",
        )?; // < 1 ms

        self.hash_glossary()?;

        // self.conn.execute_batch("VACUUM;")?; // Not needed, twice time taken than import

        Ok(YomitanZipImportResult {
            exists: true,
            load: true,
            error: None,
        })
    }

    pub fn drop_dictionary(&self, bundle_name: &str, lang: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM dictionaries WHERE bundle_name = ?1 AND lang = ?2",
            [bundle_name, lang],
        )?;

        Ok(())
    }

    pub fn make_search_db(
        &mut self,
        tokenizer: Tokenizer,
        progress_callback: impl Fn(YomitanProgress),
    ) -> Result<(), CJDicError> {
        let mut search_db = SearchDatabase::new(&mut self.conn);
        search_db.reset_db()?;
        search_db.regenerate_yomitan("main", tokenizer.clone(), &progress_callback)?;

        Ok(())
    }
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
