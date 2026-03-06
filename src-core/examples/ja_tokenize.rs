use std::env::args;

use cjdic2_core::AppService;

mod common;
use common::get_db_dir;

use crate::common::get_vibrato_dict_dir;

fn main() -> Result<(), anyhow::Error> {
    let service = AppService::new(get_db_dir(), get_vibrato_dict_dir())?;

    let mut arg_it = args();
    arg_it.next();

    println!(
        "{:#?}",
        service.ja_tokenize(arg_it.next().unwrap_or(
            "武器･防具≪エルガー武器商会≫の裏の鍛冶場を調べ「錆びた鍵」を入手します ".to_string()
        ))?
    );

    Ok(())
}
