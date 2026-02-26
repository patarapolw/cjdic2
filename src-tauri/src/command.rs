// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use cjdic2_core::{AppService, YomitanRow};
use tauri::{AppHandle, Emitter, Manager};
#[cfg(target_os = "android")]
use tauri_plugin_fs::FsExt;

#[tauri::command]
pub async fn init_yomitan(
    app: AppHandle,
    state: tauri::State<'_, AppService>,
) -> Result<(), String> {
    let folder_name = "resources/yomitan";

    // 1. Resolve the internal Asset URI
    let zip_dir = app
        .path()
        .resolve(folder_name, tauri::path::BaseDirectory::Resource)
        .map_err(|e| {
            eprintln!("{:?}", e);
            e.to_string()
        })?;

    let config = app.config();
    println!("{:#?}", app.asset_resolver());
    if let Some(r) = &config.bundle.resources {
        println!("{:?}", r);
    }

    // Embedded files cannot be read with std fs directly, not to mention read_dir
    // It need to be copied with app.fs().read(...) one-by-one, and no read_dir here
    // @link https://v2.tauri.app/plugin/file-system/
    // app.config().bundle.resources don't seem to resolve **/*
    #[cfg(target_os = "android")]
    let zip_dir = {
        let filename = "PixivLight_2026-02-24.zip";

        // 2. Define the physical destination in AppData
        let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
        let app_data_path = app_data_dir.join(filename);

        if !app_data_path.exists() {
            // 3. IMPORTANT: Use app.fs().read() to read from the APK asset
            // std::fs::read will fail with "os error 2" on Android
            let data = app
                .fs()
                .read(&zip_dir.join(filename))
                .map_err(|e| format!("Failed to read asset from APK: {}", e))?;

            // 4. Use std::fs to write to a real physical path in AppData
            if let Some(parent) = app_data_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            std::fs::write(&app_data_path, data)
                .map_err(|e| format!("Failed to write to AppData: {}", e))?;
        }

        app_data_dir
    };

    let r = state
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
        .map_err(|e| {
            eprintln!("{:?}", e);
            e.to_string()
        })?;

    if r.new_dicts.len() + r.to_be_removed_dicts.len() > 0 {
        app.request_restart();
    }

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
        .map_err(|e| {
            eprintln!("{:?}", e);
            e.to_string()
        })
}
