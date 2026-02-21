use serde::Serialize;

use crate::state;

#[derive(Serialize)]
pub struct SearchRow {
    a: i32,
    b: i32,
}

#[tauri::command]
pub fn get_db(state: tauri::State<state::AppState>) -> Result<Vec<SearchRow>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT a, b FROM test").map_err(|e| e.to_string())?;

    let rows = stmt.query_map([], |r| {
        Ok(SearchRow {
            a: r.get(0)?,
            b: r.get(1)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(|e| e.to_string())?);
    }

    Ok(out)
}
