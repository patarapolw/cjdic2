use std::time::Instant;

use anyhow::Ok;
use cjdic2_core::db::Database;

fn main() -> Result<(), anyhow::Error> {
    let db = Database::new("test.db")?;

    let start = Instant::now();
    let rs = db.yomitan().search_yomitan("擦る", "する", 10, 0)?;
    println!("{:#?}", rs);
    println!("n={}", rs.len());
    println!("[{:.2?}]", start.elapsed());

    Ok(())
}
