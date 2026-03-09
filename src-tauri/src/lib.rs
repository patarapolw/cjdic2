use std::fs::{self, create_dir_all};

use crate::command::*;
use cjdic2_core::*;
use tauri::Manager;
use tauri_plugin_fs::FsExt;

mod command;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            let config_dir = app.path().app_config_dir()?;
            create_dir_all(&config_dir)?;

            let data_dir = app.path().app_data_dir()?;
            create_dir_all(&data_dir.join("yomitan/ja"))?;

            let cache_dir = app.path().app_cache_dir()?;
            let vibrato_dic_dir = cache_dir.join("vibrato");
            create_dir_all(&vibrato_dic_dir)?;

            let vibrato_dic_path = vibrato_dic_dir.join("system.dic");
            if !vibrato_dic_path.exists() {
                let zst_path = app.path().resolve(
                    "resources/mecab-ipadic/system.dic.zst",
                    tauri::path::BaseDirectory::Resource,
                )?;

                // Use Tauri's fs abstraction — works on both Android and desktop
                let bytes = app.fs().read(&zst_path)?;

                let mut decoder = zstd::Decoder::new(bytes.as_slice())?;
                let mut out = fs::File::create(&vibrato_dic_path)?;
                std::io::copy(&mut decoder, &mut out)?;
            }

            // db_path is at config_dir;
            let service = AppService::new(&config_dir, &vibrato_dic_path)?;
            app.manage(service);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            init_yomitan,
            import_yomitan_dict,
            remove_yomitan_dict,
            search_yomitan,
            execute_sql,
            download_url,
            tokenize,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
