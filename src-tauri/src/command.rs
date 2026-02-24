// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use cjdic2_core::{AppService, YomitanRow};

#[tauri::command]
pub fn is_yomitan_setup_yet(state: tauri::State<AppService>) -> Result<bool, String> {
    state.is_yomitan_setup_yet().map_err(|e| e.to_string())
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
