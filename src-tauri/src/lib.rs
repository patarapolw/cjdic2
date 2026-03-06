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
            let app_dir = app.path().app_config_dir()?;
            create_dir_all(&app_dir)?;

            let dict_path = {
                // Resolve the resource path (returns asset://localhost/ URI on Android)
                let resource_path = app.path().resolve(
                    "resources/naist-jdic-mecab/system.dic.zst",
                    tauri::path::BaseDirectory::Resource,
                )?;

                // Use Tauri's fs abstraction — works on both Android and desktop
                let bytes = app.fs().read(&resource_path)?;

                // Write to app's cache dir, which IS a real writable filesystem path on Android
                let cache_dir = app.path().cache_dir()?;
                create_dir_all(&cache_dir)?;
                let dict_path = cache_dir.join("system.dic.zst");
                fs::write(&dict_path, &bytes)?;

                dict_path
            };

            // db_path is app_dir;
            let service = AppService::new(&app_dir, &dict_path)?;
            app.manage(service);

            let app_data_dir = app.path().app_data_dir()?;
            create_dir_all(&app_data_dir.join("yomitan/ja"))?;

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
