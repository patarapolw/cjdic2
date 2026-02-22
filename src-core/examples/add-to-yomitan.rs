use std::{
    fs::read_dir,
    path::{Path, absolute},
    time::Instant,
};

use anyhow::{Context, Ok};
use cjdic2_core::db::import_bundled_zip_file;
use rusqlite::Connection;

fn main() -> Result<(), anyhow::Error> {
    let conn = &mut Connection::open("test.db")?;
    let zip_dir = Path::new("src-tauri/resources/yomitan");
    println!("zip_dir: {:?}", absolute(zip_dir)); // Relative to workspace root

    for entry in
        read_dir(&zip_dir).with_context(|| format!("reading zip dir {}", zip_dir.display()))?
    {
        let e = entry?;
        let p = e.path();

        if p.extension().and_then(|s| s.to_str()) == Some("zip") {
            let start = Instant::now();

            println!("zip_file: {:?}", p);
            println!(
                "{:?}\n[{:.2?}]",
                import_bundled_zip_file(conn, p)?,
                start.elapsed(),
            );
        } else {
            println!("not zip_file: {:?}", p);
        }
    }
    Ok(())
}
