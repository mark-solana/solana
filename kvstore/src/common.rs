use bytes::{Bytes, BytesMut};
use std::ops::{Deref, DerefMut};

#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Key {
    pub bytes: BytesMut,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Record {
    pub key: Key,
    pub value: Option<Bytes>,
}

impl Key {
    pub fn new(bytes: impl Into<BytesMut>) -> Self {
        Key {
            bytes: bytes.into(),
        }
    }
}

impl Record {
    pub fn new(key: Key, value: Option<Bytes>) -> Self {
        Record { key, value }
    }
}

impl Deref for Key {
    type Target = BytesMut;

    fn deref(&self) -> &BytesMut {
        &self.bytes
    }
}

impl DerefMut for Key {
    fn deref_mut(&mut self) -> &mut BytesMut {
        &mut self.bytes
    }
}

impl<T> From<T> for Key
where
    T: Into<BytesMut>,
{
    fn from(bytes: T) -> Self {
        Key::new(bytes)
    }
}
