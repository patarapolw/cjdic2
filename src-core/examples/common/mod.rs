use std::path::PathBuf;

pub fn get_db_dir() -> PathBuf {
    PathBuf::from("tmp/save-db")
}
