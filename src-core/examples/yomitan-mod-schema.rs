use std::time::Instant;

use anyhow::Ok;
use cjdic2_core::db::Database;

fn main() -> Result<(), anyhow::Error> {
    let db = Database::new("test.db")?;

    let start = Instant::now();
    db.yomitan().create_schema()?;
    println!("[{:.2?}]", start.elapsed());

    Ok(())
}
