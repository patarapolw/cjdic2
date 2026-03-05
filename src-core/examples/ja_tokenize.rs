use cjdic2_core::AppService;

mod common;
use common::get_db_dir;

use crate::common::get_vibrato_dict_dir;

fn main() -> Result<(), anyhow::Error> {
    let service = AppService::new(get_db_dir(), get_vibrato_dict_dir())?;

    println!(
        "{:#?}",
        service.ja_tokenize(
            "予約キャンセル・変更は【配信日前日の19時頃】までに行ってください。".to_string()
        )?
    );

    Ok(())
}
