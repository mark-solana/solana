use crate::components::{Key, LogFile, Value};
use crate::Config;

use bytes::BufMut;

use crc::crc32;

use evmap::WriteHandle;

use specs::prelude::*;

use std::io::{self, Write};
use std::mem::size_of;

pub struct Writer {
    w_map: WriteHandle<Key, Value>,
}

impl Writer {
    pub fn new(w_map: WriteHandle<Key, Value>) -> Self {
        Writer { w_map }
    }
}

//impl<'a> System<'a> for Writer {
//type SystemData = (Read<'a, Config>, ReadStorage<'a, LogFile>);

//fn run(&mut self, (config, files): Self::SystemData) {
//for log_file in files.join() {
//if !config.in_memory {
//let writer = &log_file.file;

//self.r_map.for_each(|key, values| {
//log(writer, key, &values[0]).expect("I/O Error in log");
//});
//}
//}
//}
//}

fn log<W>(mut writer: W, key: &Key, value: &Value) -> io::Result<()>
where
    W: Write,
{
    let header_size = size_of::<usize>() + size_of::<u32>();
    let size = header_size + key.len() + value.bytes.as_ref().map(|b| b.len()).unwrap_or(0);

    let mut buf = Vec::with_capacity(size);
    let (header, payload) = buf.split_at_mut(header_size);
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

    writer.write_all(&buf)?;

    Ok(())
}
