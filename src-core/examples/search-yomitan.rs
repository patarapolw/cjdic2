use std::time::Instant;

use anyhow::Ok;
use cjdic2_core::AppService;

mod common;
use common::get_db_dir;

fn main() -> Result<(), anyhow::Error> {
    let service = AppService::new(get_db_dir())?;

    let start = Instant::now();
    let rs = service.search_yomitan("擦る", "する", 10, 0)?;
    println!("{:#?}", rs);
    println!("n={}", rs.len());
    println!("[{:.2?}]", start.elapsed());

    Ok(())
}
