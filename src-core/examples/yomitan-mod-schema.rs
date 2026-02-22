use std::time::Instant;

use anyhow::Ok;
use cjdic2_core::db::Database;

mod common;
use common::get_db_path;

fn main() -> Result<(), anyhow::Error> {
    let db = Database::new(get_db_path())?;

    let start = Instant::now();
    db.yomitan().create_schema()?;
    println!("[{:.2?}]", start.elapsed());

    Ok(())
}
