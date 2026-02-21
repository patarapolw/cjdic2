use crate::command::*;
use cjdic2_core::*;
use tauri::Manager;

mod command;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let cfg_dir = app.path().app_config_dir();
            let app_dir = cfg_dir.unwrap_or_else(|_| {
                std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
            });
            std::fs::create_dir_all(&app_dir)?;

            let db_path = app_dir.join("cjdic.db");
            let service = AppService::new(&db_path)?;
            app.manage(service);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![add_entry, list_entries])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app_handle, event| match event {
            // On exit handler should be avoided for mobile
            // and rusqlite already closes gracefully by default
            tauri::RunEvent::ExitRequested { .. } => {}
            _ => {}
        });
}
