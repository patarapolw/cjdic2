use std::{
    cell::Cell,
    fs::{File, remove_file},
    path::PathBuf,
};

use cjdic2_core::{
    AppService, CJDicError, SqlParam, TokenizeSegment, YomitanDictEntry, YomitanRow,
    YomitanZipImportResult,
};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_http::reqwest::Client;
use tokio::io::AsyncWriteExt;
use zip::ZipArchive;

#[tauri::command]
pub async fn list_yomitan_dict(
    state: tauri::State<'_, AppService>,
) -> Result<Vec<YomitanDictEntry>, CJDicError> {
    state.list_yomitan_dict()
}

#[derive(Deserialize)]
pub struct YomitanDownloadDictEntry {
    url: String,
    filepath: String,
}

#[tauri::command]
pub async fn init_yomitan(
    app: AppHandle,
    dicts: Vec<YomitanDownloadDictEntry>,
    lang: String,
    state: tauri::State<'_, AppService>,
) -> Result<(), CJDicError> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| CJDicError::Error(e.to_string()))?;

    for d in dicts.iter().clone() {
        let zip_path = app_dir.join(&d.filepath);
        if zip_path.exists() && File::open(zip_path).is_ok_and(|f| ZipArchive::new(f).is_ok()) {
            continue;
        }

        download_url_local(
            d.url.clone(),
            app_dir.clone(),
            d.filepath.clone(),
            |progress| {
                app.emit("download-url-progress", progress).unwrap();
            },
        )
        .await
        .map_err(|e| CJDicError::Error(e.to_string()))?;
    }

    let zip_files: Vec<PathBuf> = dicts
        .into_iter()
        .map(|d| app_dir.join(&d.filepath))
        .collect();

    let update_count = Cell::new(0usize);

    state.load_yomitan_zip_dir(
        zip_files,
        lang.as_str(),
        |r| {
            update_count.set(update_count.get() + r.new_dicts.len() + r.to_be_removed_dicts.len());
            app.emit("load-yomitan-dir", r).unwrap();
        },
        |r| {
            update_count.set(update_count.get() + 1);
            app.emit("yomitan-import-progress", r).unwrap();
        },
    )?;

    if update_count.get() > 0 {
        println!("Request app restart?",);
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

    let mut writer = state.get_yomitan_writer(|r| {
        app.emit("yomitan-import-progress", r).unwrap();
    })?;
    AppService::import_yomitan_zip_file(&mut writer, &data_dir, lang, |r| {
        app.emit("yomitan-import-progress", r).unwrap();
    })
}

#[tauri::command]
pub async fn remove_yomitan_dict(
    app: AppHandle,
    bundle_name: &str,
    lang: &str,
    state: tauri::State<'_, AppService>,
) -> Result<(), CJDicError> {
    let mut writer = state.get_yomitan_writer(|r| {
        app.emit("yomitan-import-progress", r).unwrap();
    })?;
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

async fn download_url_local<Callback>(
    url: String,
    data_dir: PathBuf,
    filepath: String,
    callback: Callback,
) -> Result<bool, CJDicError>
where
    Callback: Fn(DownloadProgress),
{
    let client = Client::new();
    let file_pf = data_dir.join(filepath.as_str());

    let temp_pf = data_dir.join(format!("{}.dl-tmp", filepath));
    let temp_path = temp_pf.as_path();
    // Check existing partial download size
    let existing_size = tokio::fs::metadata(temp_path)
        .await
        .map(|m| m.len())
        .unwrap_or(0);

    let mut request = client.get(url.as_str());
    // Request only the remaining bytes
    if existing_size > 0 {
        request = request.header("Range", format!("bytes={}-", existing_size));
    }

    // Making the asynchronous GET request
    let mut response = request
        .send()
        .await
        .map_err(|e| CJDicError::Error(e.to_string()))?;

    // Check if the request was successful
    if response.status().is_success() {
        let content_length = response.content_length().unwrap_or(0);
        let mut file = if existing_size > 0 {
            tokio::fs::OpenOptions::new()
                .append(true)
                .open(temp_path)
                .await
                .map_err(|e| CJDicError::Error(e.to_string()))?
        } else {
            tokio::fs::File::create(&file_pf)
                .await
                .map_err(|e| CJDicError::Error(e.to_string()))?
        };

        // Downloading the file in chunks
        let mut downloaded: u64 = 0;
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|e| CJDicError::Error(e.to_string()))?
        {
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;

            callback(DownloadProgress {
                url: url.clone(),
                filepath: filepath.clone(),
                content_length,
                downloaded,
            });
        }

        // Verify download completeness
        if content_length > 0 && downloaded < content_length {
            // Delete incomplete file
            drop(file); // ensure file handle is released before deleting
            remove_file(&file_pf).ok();
            return Err(CJDicError::Error(format!(
                "Incomplete download: expected {} bytes, got {}",
                content_length, downloaded
            )));
        }

        println!("File downloaded successfully to {:?}", &file_pf);
    } else {
        eprintln!("Failed to download file: {:?}", response.status());
    }

    Ok(true)
}

#[tauri::command]
pub async fn download_url(
    app: AppHandle,
    url: String,
    filepath: String,
) -> Result<bool, CJDicError> {
    download_url_local(
        url,
        app.path()
            .app_data_dir()
            .map_err(|e| CJDicError::Error(e.to_string()))?,
        filepath,
        |progress| {
            app.emit("download-url-progress", progress).unwrap();
        },
    )
    .await
    .map_err(|e| CJDicError::Error(e.to_string()))
}

#[tauri::command]
pub async fn tokenize(
    text: String,
    state: tauri::State<'_, AppService>,
) -> Result<Vec<TokenizeSegment>, CJDicError> {
    state.tokenize(text)
}
