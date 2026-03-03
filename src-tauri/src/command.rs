use std::{borrow::Cow, fs::File, io::Write, path::PathBuf};

use cjdic2_core::{
    AppService, CJDicError, SqlParam, YomitanRow, YomitanZipImportResult, ZipSource,
};
use reqwest::Client;
use serde::Serialize;
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
            .map_err(|e| CJDicError::Error(e.to_string()))?;

        // Manifest is needed, as Android bundled fs can't read_dir
        let files: Vec<String> =
            serde_json::from_str(&app.fs().read_to_string(zip_dir.join("manifest.json"))?)?;

        let zip_files = files
            .iter()
            .map(|f| BundledZip::new(app.clone(), &format!("{folder_name}/{f}")))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| CJDicError::Error(e.to_string()))?;

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
pub async fn import_yomitan_dict(
    app: AppHandle,
    bundle_name: &str,
    lang: &str,
    state: tauri::State<'_, AppService>,
) -> Result<YomitanZipImportResult, CJDicError> {
    let mut data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| CJDicError::Error(e.to_string()))?;
    data_dir.push("yomitan");
    data_dir.push(lang);
    data_dir.push(bundle_name);

    let mut writer = state.get_yomitan_writer()?;
    AppService::import_yomitan_zip_file(&mut writer, &data_dir, lang, |r| {
        app.emit("yomitan-import-progress", r).unwrap();
    })
}

#[tauri::command]
pub async fn remove_yomitan_dict(
    bundle_name: &str,
    lang: &str,
    state: tauri::State<'_, AppService>,
) -> Result<(), CJDicError> {
    let mut writer = state.get_yomitan_writer()?;
    AppService::remove_yomitan_dictionary(&mut writer, bundle_name, lang)
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

#[derive(Serialize, Clone)]
struct DownloadProgress {
    url: String,
    filepath: String,
    content_length: u64,
    downloaded: u64,
}

#[tauri::command]
pub async fn download_url(
    app: AppHandle,
    url: String,
    filepath: String,
) -> Result<bool, CJDicError> {
    let client = Client::new();
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| CJDicError::Error(e.to_string()))?;
    let file_pf = data_dir.join(filepath.as_str());

    // Making the asynchronous GET request
    let mut response = client
        .get(url.as_str())
        .send()
        .await
        .map_err(|e| CJDicError::Error(e.to_string()))?;

    // Check if the request was successful
    if response.status().is_success() {
        let content_length = response.content_length().unwrap_or(0);
        let mut file = File::create(&file_pf).map_err(|e| CJDicError::Error(e.to_string()))?;

        // Downloading the file in chunks
        let mut downloaded: u64 = 0;
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|e| CJDicError::Error(e.to_string()))?
        {
            file.write_all(&chunk)?;
            downloaded += chunk.len() as u64;

            app.emit(
                "download-url-progress",
                DownloadProgress {
                    url: url.clone(),
                    filepath: filepath.clone(),
                    content_length,
                    downloaded,
                },
            )
            .unwrap();
        }
        println!("File downloaded successfully to {:?}", &file_pf);
    } else {
        eprintln!("Failed to download file: {:?}", response.status());
    }

    Ok(true)
}
