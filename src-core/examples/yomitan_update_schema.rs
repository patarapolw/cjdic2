use anyhow::Ok;
use cjdic2_core::{AppService, Timer};

mod common;
use common::get_db_dir;

use crate::common::get_vibrato_dict_dir;

fn main() -> Result<(), anyhow::Error> {
    let service = AppService::new(get_db_dir(), get_vibrato_dict_dir())?;

    {
        let _timer = Timer::new("update schema".to_string());
        service.get_yomitan_writer(|p| {
            println!("{:?}", p);
        })?;
    }

    Ok(())
}
