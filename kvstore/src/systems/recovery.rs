use crate::components::{Key, LogFile, Value};
use crate::Config;

use byteorder::{LittleEndian, ReadBytesExt};

use bytes::{Buf, BufMut};

use crc::crc32;

use evmap::WriteHandle;

use specs::prelude::*;

use std::io::{self, Write as _};
use std::mem::size_of;
use std::sync::Mutex;

const HEADER_SIZE: usize = size_of::<usize>() + size_of::<u32>();

pub struct Recovery;

impl<'a> System<'a> for Recovery {
    type SystemData = (
        Read<'a, Config>,
        WriteExpect<'a, Mutex<WriteHandle<Key, Value>>>,
        ReadExpect<'a, LogFile>,
    );

    fn run(&mut self, (config, map_mutex, log_file): Self::SystemData) {
        if !config.in_memory {
            let reader = &log_file.file;
            let mut w_map = map_mutex.try_lock().unwrap();

            match recover(reader, &mut w_map) {
                Ok(_) => println!("recovered successfully"),
                Err(e) => eprintln!("I/O Error: {}", e),
            }
        }
    }
}

fn recover<R>(mut reader: R, w_map: &mut WriteHandle<Key, Value>) -> io::Result<()>
where
    R: io::Read,
{
    unimplemented!();

    let header_buf = &mut [0; HEADER_SIZE][..];

    //loop {
    //let len = header_buf.read_u64::<LittleEndian>();
    //let crc = header_buf.get_u32_le();
    //}

    Ok(())
}

fn log<W>(mut writer: W, key: &Key, value: &Value) -> io::Result<()>
where
    W: io::Write,
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
