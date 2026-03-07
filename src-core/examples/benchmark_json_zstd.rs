use anyhow::Ok;

mod common;
use common::get_db_dir;
use rusqlite::{Connection, params};

fn benchmark_dict_sizes(samples: &[String]) {
    // 112640; // 110KB default gives the best 5.8x compression
    for size in [16384, 65536, 112640, 524288] {
        let total: usize = samples.iter().map(|s| s.len()).sum();
        let min_total = size * 100;

        println!(
            "samples={} total={}KB min_required={}KB",
            samples.len(),
            total / 1024,
            min_total / 1024
        );

        let dict = zstd::dict::from_samples(samples, size).unwrap();
        let compressed_sizes: usize = samples
            .iter()
            .map(|s| {
                zstd::bulk::Compressor::with_dictionary(3, &dict)
                    .unwrap()
                    .compress(s.as_bytes())
                    .unwrap()
                    .len()
            })
            .sum();
        let original_sizes: usize = samples.iter().map(|s| s.len()).sum();
        println!(
            "dict={}KB ratio={:.1}x",
            size / 1024,
            original_sizes as f64 / compressed_sizes as f64
        );
    }
}

fn main() -> Result<(), anyhow::Error> {
    let conn = Connection::open(get_db_dir().join("yomitan.db"))?;

    let max_id: i64 = conn.query_row("SELECT MAX(id) FROM glossaries", [], |r| r.get(0))?;
    let step = max_id / 1000;

    let samples: Vec<String> = conn
        .prepare("SELECT content FROM glossaries WHERE id % ?1 = 0 LIMIT 1000")?
        .query_map(params![step], |r| r.get::<_, String>(0))?
        .filter_map(|r| r.ok())
        .collect();

    println!("{} {}", step, samples.len());

    benchmark_dict_sizes(&samples);

    Ok(())
}
