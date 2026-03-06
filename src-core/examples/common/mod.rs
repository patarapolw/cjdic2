use std::path::PathBuf;

pub fn get_db_dir() -> PathBuf {
    PathBuf::from("tmp/save-db")
}

pub fn get_vibrato_dict_dir() -> PathBuf {
    PathBuf::from("src-tauri/resources/naist-jdic-mecab/system.dic.zst")
}
