use anyhow::{Ok, Result};

mod common;
use common::get_db_dir;
use rusqlite::Connection;

#[allow(unused)]
fn benchmark_decompressor(conn: &Connection) -> Result<()> {
    let zstd_dict: Vec<u8> =
        conn.query_row("SELECT value FROM meta WHERE key = 'zstd_dict'", [], |r| {
            r.get(0)
        })?;

    let samples: Vec<Vec<u8>> = conn
        .prepare("SELECT b FROM glossaries WHERE id % 2111 = 0 LIMIT 100")?
        .query_map([], |r| r.get::<_, Vec<u8>>(0))?
        .filter_map(|r| r.ok())
        .collect();

    // benchmark: new decompressor per call
    let start = std::time::Instant::now();
    for sample in &samples {
        let mut decompressor = zstd::bulk::Decompressor::with_dictionary(&zstd_dict)?;
        let _ = decompressor.decompress(sample, 1024 * 1024)?;
    }

    // new decompressor per call: 75.002µs avg
    let per_call = start.elapsed() / samples.len() as u32;
    println!("new decompressor per call: {:?} avg", per_call);

    // benchmark: reuse decompressor
    let start = std::time::Instant::now();
    let mut decompressor = zstd::bulk::Decompressor::with_dictionary(&zstd_dict)?;
    for sample in &samples {
        let _ = decompressor.decompress(sample, 1024 * 1024)?;
    }
    // reused decompressor:       44.231µs avg
    let per_call = start.elapsed() / samples.len() as u32;
    println!("reused decompressor:       {:?} avg", per_call);

    Ok(())
}

// measure your actual maximum
fn find_max_content_size(conn: &Connection) -> Result<()> {
    // load dictionary once
    let dict = conn.query_row("SELECT value FROM meta WHERE key = 'zstd_dict'", [], |r| {
        r.get::<_, Vec<u8>>(0)
    })?;

    let (avg, max): (f64, i64) = conn.query_row(
        "SELECT avg(length(b)), max(length(b)) FROM glossaries",
        [],
        |r| rusqlite::Result::Ok((r.get(0)?, r.get(1)?)),
    )?;
    println!("avg compressed: {:.0} bytes", avg);
    println!("max compressed: {} bytes", max);

    // decompress the largest one to find actual max decompressed size
    let largest: Vec<u8> = conn.query_row(
        "SELECT b FROM glossaries
        WHERE length(b) = (SELECT max(length(b)) FROM glossaries)",
        [],
        |r| r.get(0),
    )?;

    // use a generous cap just to measure
    let mut decompressor = zstd::bulk::Decompressor::with_dictionary(&dict)?;
    let decompressed = decompressor.decompress(&largest, 100 * 1024 * 1024)?;
    println!("max decompressed: {} bytes", decompressed.len());

    Ok(())
}

fn main() -> Result<(), anyhow::Error> {
    let conn = Connection::open(get_db_dir().join("yomitan-glossary.db"))?;
    find_max_content_size(&conn)?;
    Ok(())
}
