use anyhow::Ok;
use cjdic2_core::db::search_yomitan;
use rusqlite::Connection;

fn main() -> Result<(), anyhow::Error> {
    let conn = &mut Connection::open("test.db")?;
    let rs = search_yomitan(conn, "食べる", "食べる", 5, 0)?;
    println!("{:#?}", rs);

    Ok(())
}
