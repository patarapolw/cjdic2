use std::sync::{Mutex, OnceLock};

pub use cjdic2_core::{AppService, Entry};

static SERVICE: OnceLock<Mutex<AppService>> = OnceLock::new();

pub fn init(db_path: String) -> Result<(), String> {
    let service = AppService::new(db_path).map_err(|e| e.to_string())?;

    SERVICE
        .set(Mutex::new(service))
        .map_err(|_| "Already initialized".to_string())?;

    Ok(())
}

pub fn add_entry(name: String) -> Result<(), String> {
    let service = SERVICE.get().ok_or("Not initialized")?;
    let guard = service.lock().unwrap();

    guard
        .add_entry(&name, "definition")
        .map_err(|e| e.to_string())
}

pub fn list_entries() -> Result<Vec<Entry>, String> {
    let service = SERVICE.get().ok_or("Not initialized")?;
    let guard = service.lock().unwrap();

    guard.list_entries().map_err(|e| e.to_string())
}
