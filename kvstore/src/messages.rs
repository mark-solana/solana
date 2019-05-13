use actix::Message;

use bytes::Bytes;

pub struct Write<'a>(pub &'a[u8]);

pub struct
