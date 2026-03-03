use anyhow::Ok;
use cjdic2_core::{AppService, Timer};

mod common;
use common::get_db_dir;

fn main() -> Result<(), anyhow::Error> {
    let service = AppService::new(get_db_dir())?;

    {
        let _timer = Timer::new("remove one".to_string());
        let mut writer = service.get_yomitan_writer()?;
        AppService::remove_yomitan_dictionary(&mut writer, "JMnedict")?;
    }

    Ok(())
}
