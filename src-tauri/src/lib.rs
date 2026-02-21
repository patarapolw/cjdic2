use tauri::Manager;
use rusqlite::Connection;
use std::sync::Mutex;

mod db;
mod state;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let cfg_dir = app.path().app_config_dir();
            let app_dir = cfg_dir
                .unwrap_or_else(|_| std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")));

            std::fs::create_dir_all(&app_dir)?;

            let db_path = app_dir.join("cjdic.db");
            let is_new = !db_path.exists();

            println!("{}", cjdic2_core::add(1, 2));

            let conn = Connection::open(&db_path)?;

            if is_new {
                conn.execute_batch("CREATE TABLE test (a, b);")?;
                let mut stmt = conn.prepare("INSERT INTO test VALUES (?1, ?2)")?;
                stmt.insert([1, 2])?;
                stmt.insert([3, 4])?;
            }

            app.manage(state::AppState{
                conn: Mutex::new(conn),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet, db::get_db])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app_handle, event| match event {
            // On exit handler should be avoided for mobile
            // and rusqlite already closes gracefully by default
            tauri::RunEvent::ExitRequested { .. } => {}
            _ => {}
        });
}
