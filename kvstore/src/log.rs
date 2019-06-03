use crate::common::Record;
use crc::crc32::{Digest, Hasher32, IEEE};

pub mod components;
pub mod systems;

pub const DEFAULT_LOG_SIZE: usize = 64 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub max_file_size: usize,
    pub fsync: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogRecord {
    pub checksum: u32,
    pub record: Record,
}

impl LogRecord {
    pub fn new(record: Record) -> Self {
        let mut digest = Digest::new(IEEE);
        digest.write(&record.key.bytes);
        if let Some(ref bytes) = record.value {
            digest.write(&bytes);
        }

        let checksum = digest.sum32();

        LogRecord { checksum, record }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            max_file_size: DEFAULT_LOG_SIZE,
            fsync: true,
        }
    }
}
