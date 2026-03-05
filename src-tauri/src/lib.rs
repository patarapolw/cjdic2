use std::fs::create_dir_all;

use crate::command::*;
use cjdic2_core::*;
use tauri::Manager;

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

            // db_path is app_dir;
            let db_dir = app_dir.as_path();
            let service = AppService::new(&db_dir)?;
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
