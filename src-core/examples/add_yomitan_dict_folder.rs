use std::{
    fs::read_dir,
    path::{Path, absolute},
    time::Instant,
};

use anyhow::{Context, Ok};
use cjdic2_core::AppService;

mod common;
use common::get_db_dir;

fn main() -> Result<(), anyhow::Error> {
    let service = AppService::new(get_db_dir())?;
    let mut writer = service.get_yomitan_writer()?;

    let zip_dir = Path::new("tmp/yomitan");
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
                "{:?}",
                AppService::import_yomitan_zip_file(&mut writer, p, "ja")?,
            );
            println!("[{:.2?}]", start.elapsed());
        } else {
            println!("not zip_file: {:?}", p);
        }
    }
    Ok(())
}
