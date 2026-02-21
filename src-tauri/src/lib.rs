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
            println!("{:?}", db_path);

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
            tauri::RunEvent::ExitRequested { .. } => {
                // api.prevent_exit();
                // let handle = app_handle.clone();

                // if let Some(state) = app_handle.try_state::<state::AppState>() {
                //     if let Ok(conn) = state.conn.lock() {
                //         if let Err(e) = (|| -> Result<(), rusqlite::Error> {
                //             conn.execute_batch("PRAGMA optimize;")?;
                //             conn.execute("PRAGMA wal_checkpoint(FULL);", [])?;

                //             let (_busy, _log, _checkpointed): (i32, i32, i32) =
                //             conn.query_row("PRAGMA wal_checkpoint(FULL);", [], |row| {
                //                 Ok((row.get(0)?, row.get(1)?, row.get(2)?))
                //             })?;
                //             Ok(())
                //         })() {
                //             eprintln!("Shutdown SQL failed: {e}");
                //         }
                //     }
                // }

                // handle.exit(0);
            }
            _ => {}
        });
}
