use std::time::Instant;

use anyhow::Ok;
use cjdic2_core::db::create_schema;
use rusqlite::Connection;

fn main() -> Result<(), anyhow::Error> {
    let conn = &mut Connection::open("test.db")?;

    let start = Instant::now();
    create_schema(conn)?;
    println!("[{:.2?}]", start.elapsed());

    Ok(())
}
