// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use cjdic2_core::{AppService, YomitanRow};
use tauri::{AppHandle, Emitter, Manager, path::BaseDirectory};

#[tauri::command]
pub async fn init_yomitan(
    app: AppHandle,
    state: tauri::State<'_, AppService>,
) -> Result<(), String> {
    let zip_dir = app
        .path()
        .resolve("resources/yomitan", BaseDirectory::Resource)
        .map_err(|e| e.to_string())?;
    // println!("{:?}", zip_dir.as_path());

    state
        .load_yomitan_zip_dir(
            zip_dir,
            "ja",
            |r| {
                app.emit("load-yomitan-dir", r).unwrap();
            },
            |r| {
                app.emit("yomitan-import-progress", r).unwrap();
            },
        )
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn search_yomitan(
    q_term: &str,
    q_reading: &str,
    limit: u32,
    offset: u32,
    state: tauri::State<'_, AppService>,
) -> Result<Vec<YomitanRow>, String> {
    state
        .search_yomitan(q_term, q_reading, limit, offset)
        .map_err(|e| e.to_string())
}
