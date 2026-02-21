// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use cjdic2_core::{AppService, Entry};

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
