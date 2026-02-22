use std::time::Instant;

use anyhow::Ok;
use cjdic2_core::db::Database;

mod common;
use common::get_db_path;

fn main() -> Result<(), anyhow::Error> {
    let db = Database::new(get_db_path())?;

    let start = Instant::now();
    let rs = db.yomitan().search_yomitan("擦る", "する", 10, 0)?;
    println!("{:#?}", rs);
    println!("n={}", rs.len());
    println!("[{:.2?}]", start.elapsed());

    Ok(())
}
