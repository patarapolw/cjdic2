use std::path::PathBuf;

pub fn get_db_dir() -> PathBuf {
    PathBuf::from(r"C:\Users\HP\AppData\Roaming\cc.polv.cjdic2")
}

pub fn get_vibrato_dict_dir() -> PathBuf {
    PathBuf::from("src-tauri/resources/naist-jdic-mecab/system.dic.zst")
}
