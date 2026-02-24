mod db;
mod error;
mod service;

pub use db::{YomitanRow, YomitanZipImportProgress, YomitanZipImportResult};
pub use error::CJDicError;
pub use service::AppService;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
