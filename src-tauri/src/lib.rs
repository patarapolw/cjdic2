use std::fs::create_dir_all;

use crate::command::*;
use cjdic2_core::*;
use tauri::Manager;

mod command;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_dir = app.path().app_config_dir()?;
            create_dir_all(&app_dir)?;

            // db_path is app_dir;
            let db_dir = app_dir.as_path();
            // let db_dir = Path::new(r"D:\Projects\cjdic2\tmp\save-db");
            let service = AppService::new(&db_dir)?;
            app.manage(service);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            is_yomitan_setup_yet,
            search_yomitan
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app_handle, event| match event {
            // On exit handler should be avoided for mobile
            // and rusqlite already closes gracefully by default
            tauri::RunEvent::ExitRequested { .. } => {}
            _ => {}
        });
}
