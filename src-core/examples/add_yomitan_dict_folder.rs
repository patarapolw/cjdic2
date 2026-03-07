use std::{
    fs::read_dir,
    path::{Path, absolute},
};

use anyhow::{Context, Ok};
use cjdic2_core::{AppService, Timer};

mod common;
use common::get_db_dir;

use crate::common::get_vibrato_dict_dir;

fn main() -> Result<(), anyhow::Error> {
    let service = AppService::new(get_db_dir(), get_vibrato_dict_dir())?;
    let mut writer = service.get_yomitan_writer(|p| {
        println!("{:?}", p);
    })?;

    let zip_dir = Path::new("src-tauri/resources/yomitan");
    println!("zip_dir: {:?}", absolute(zip_dir)); // Relative to workspace root

    for entry in
        read_dir(&zip_dir).with_context(|| format!("reading zip dir {}", zip_dir.display()))?
    {
        let e = entry?;
        let p = e.path();

        if p.extension().and_then(|s| s.to_str()) == Some("zip") {
            let _timer = Timer::new(format!("zip_file: {:?}", p));

            println!(
                "{:?}",
                AppService::import_yomitan_zip_file(&mut writer, &p, "ja", |progress| {
                    println!("{:?}", progress);
                })?,
            );
        } else {
            println!("not zip_file: {:?}", p);
        }
    }
    Ok(())
}
