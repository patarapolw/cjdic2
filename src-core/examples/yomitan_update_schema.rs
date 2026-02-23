

use std::time::Instant;

use anyhow::Ok;
use cjdic2_core::AppService;

mod common;
use common::get_db_dir;

fn main() -> Result<(), anyhow::Error> {
    let service = AppService::new(get_db_dir())?;

    let start = Instant::now();
    service.get_yomitan_writer()?;
    println!("[{:.2?}]", start.elapsed());

    Ok(())
}
