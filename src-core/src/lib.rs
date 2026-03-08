#[macro_use]
extern crate enum_map;

mod db;
mod error;
mod service;
mod util;

pub use db::{YomitanProgress, YomitanRow, YomitanZipImportResult, ZipSource};
pub use error::CJDicError;
pub use service::{AppService, SqlParam, TokenizeSegment};
pub use util::Timer;

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
