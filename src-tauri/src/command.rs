// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use cjdic2_core::{AppService, Entry, db::YomitanRow};

#[tauri::command]
pub fn add_entry(name: &str, state: tauri::State<AppService>) -> Result<(), String> {
    state
        .add_entry(name, "definition")
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_entries(state: tauri::State<AppService>) -> Result<Vec<Entry>, String> {
    state.list_entries().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn search_yomitan(
    q_term: &str,
    q_reading: &str,
    limit: u32,
    offset: u32,
    state: tauri::State<AppService>,
) -> Result<Vec<YomitanRow>, String> {
    state
        .search_yomitan(q_term, q_reading, limit, offset)
        .map_err(|e| e.to_string())
}
