use crate::components::{Key, LogFile, Value};
use crate::Config;

use bytes::BufMut;

use crc::crc32;

use specs::prelude::*;

use shred::SetupHandler;

use std::fs::OpenOptions;
use std::io::{self, Write};
use std::mem::size_of;
use std::path::PathBuf;
use std::sync::Mutex;

mod recovery;

//  payload len + crc + key len
const HEADER_SIZE: usize = size_of::<u64>() + size_of::<u32>() + size_of::<u64>();
const LOG_FILE: &'static str = "write-ahead.log";

pub struct Logging;

pub struct Setup;

#[derive(Serialize, Deserialize)]
pub struct LogRecord {
    crc: u32,
    key: Key,
    value: Value,
}

impl SetupHandler<LogFile> for Setup {
    fn setup(res: &mut Resources) {
        let path = res.fetch::<PathBuf>().clone().join(LOG_FILE);
        let opts = open_options();

        let file = opts.open(&path).expect("Couldn't open log");

        res.insert(LogFile { file, path });

        let (r_map, w_map) = evmap::new::<Key, Value>();

        res.insert(r_map.factory());
        res.insert(Mutex::new(w_map));
    }
}

impl<'a> System<'a> for Logging {
    type SystemData = (
        Read<'a, Config>,
        Read<'a, LogFile, Setup>,
        ReadStorage<'a, Key>,
        ReadStorage<'a, Value>,
    );

    fn run(&mut self, (config, log_file, keys, values): Self::SystemData) {
        if !config.in_memory {
            for (key, value) in (&keys, &values).join() {
                let writer = &log_file.file;

                let mut digest = crc32::Digest::new(crc32::IEEE);

                let res = log(writer, key, value);

                if let Err(e) = res {
                    panic!("I/O Error in the log: {}", e);
                }
            }
        }
    }
}

fn open_options() -> OpenOptions {
    let mut opts = OpenOptions::new();

    opts.create(true).write(true).read(true);

    opts
}

fn log<W>(mut writer: W, key: &Key, value: &Value) -> io::Result<()>
where
    W: Write,
{
    let size = HEADER_SIZE + key.len() + value.bytes.len();

    let mut buf = Vec::with_capacity(size);
    let (header, payload) = buf.split_at_mut(HEADER_SIZE);
    let (mut header, mut payload) = (io::Cursor::new(header), io::Cursor::new(payload));

    payload.put(&key.bytes);

    if let Some(ref bytes) = value.bytes {
        payload.put(&bytes[..]);
    } else {
        payload.put(&[0u8][..]);
    }

    let len = payload.position();

    header.put_u64_le(len);
    header.put_u32_le(crc32::checksum_ieee(payload.get_ref()));
    header.put_u64_le(key.bytes.len() as u64);

    writer.write_all(&buf)?;

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::components::{arb_key, arb_value};
    use evmap::WriteHandle;
    use proptest::prelude::*;
    use tempfile::TempDir;

    proptest! {
        #[test]
        fn test_log(key in arb_key(24), value in arb_value(48)) {
            let mut log = Logging;
            let temp_dir = TempDir::new().unwrap();
            let root = temp_dir.path().to_path_buf();

            let mut world = World::new();
            world.res.insert(root.clone());

            log.run_now(&mut world.res);
            world.create_entity().with(key).with(value).build();
            let _ = world.res.try_fetch::<Mutex<WriteHandle<Key, Value>>>().expect("map creation");
            let log_file = world.res.fetch::<LogFile>();

            assert!(log_file.file.metadata().unwrap().len() >= HEADER_SIZE as usize);
        }
    }
}
