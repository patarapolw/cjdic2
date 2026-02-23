use std::time::Instant;

use anyhow::Ok;
use cjdic2_core::AppService;

mod common;
use common::get_db_dir;

fn main() -> Result<(), anyhow::Error> {
    let service = AppService::new(get_db_dir())?;

    let start = Instant::now();
    let mut writer = service.get_yomitan_writer()?;
    AppService::remove_yomitan_dictionary(&mut writer, "JMnedict")?;
    println!("[{:.2?}]", start.elapsed());

    Ok(())
}
