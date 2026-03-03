use std::{borrow::Cow, path::PathBuf};

use cjdic2_core::{AppService, CJDicError, SqlParam, YomitanRow, ZipSource};
use tauri::{AppHandle, Emitter, Manager, Runtime, path::BaseDirectory};
use tauri_plugin_fs::FsExt;

pub struct BundledZip<R: Runtime> {
    pub name: String,
    pub path: PathBuf, // resolved asset://localhost/... path on Android
    pub app: AppHandle<R>,
}

impl<R: Runtime> BundledZip<R> {
    pub fn new(app: AppHandle<R>, relative_path: &str) -> Result<Self, tauri::Error> {
        let name = relative_path
            .split('/')
            .last()
            .unwrap_or(relative_path)
            .to_string();

        let path = app.path().resolve(relative_path, BaseDirectory::Resource)?;

        Ok(Self { name, path, app })
    }
}

impl<R: Runtime> ZipSource for BundledZip<R> {
    fn file_name(&self) -> &str {
        &self.name
    }

    fn bytes(&self) -> std::io::Result<Cow<'_, [u8]>> {
        self.app
            .fs()
            .read(&self.path) // PathBuf implements Into<FilePath>
            .map(Cow::Owned)
    }
}

#[tauri::command]
pub async fn init_yomitan(
    app: AppHandle,
    state: tauri::State<'_, AppService>,
) -> Result<(), CJDicError> {
    let mut new_dicts: Vec<String> = vec![];
    let mut to_be_removed_dicts: Vec<String> = vec![];

    for lang in vec!["ja"] {
        let folder_name = format!("resources/yomitan/{lang}");
        let folder_name = folder_name.as_str();

        let zip_dir = app
            .path()
            .resolve(folder_name, tauri::path::BaseDirectory::Resource)
            .map_err(|e| CJDicError::AnyhowError(e.to_string()))?;

        // Manifest is needed, as Android bundled fs can't read_dir
        let files: Vec<String> =
            serde_json::from_str(&app.fs().read_to_string(zip_dir.join("manifest.json"))?)?;

        let zip_files = files
            .iter()
            .map(|f| BundledZip::new(app.clone(), &format!("{folder_name}/{f}")))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| CJDicError::AnyhowError(e.to_string()))?;

        if zip_files.is_empty() {
            continue;
        }

        let mut r = state.load_yomitan_zip_dir(
            zip_files,
            lang,
            |r| {
                app.emit("load-yomitan-dir", r).unwrap();
            },
            |r| {
                app.emit("yomitan-import-progress", r).unwrap();
            },
        )?;

        new_dicts.append(&mut r.new_dicts);
        to_be_removed_dicts.append(&mut r.to_be_removed_dicts);
    }

    if new_dicts.len() + to_be_removed_dicts.len() > 0 {
        // Cleanup doesn't appear to help even after app restart
        // state.cleanup_yomitan_writer()?;

        // Request app restart anyway
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
) -> Result<Vec<YomitanRow>, CJDicError> {
    state.search_yomitan(q_term, q_reading, limit, offset)
}

#[tauri::command]
pub async fn execute_sql(
    sql: String,
    params: Vec<SqlParam>,
    state: tauri::State<'_, AppService>,
) -> Result<serde_json::Value, CJDicError> {
    state.execute_sql(sql, params)
}
